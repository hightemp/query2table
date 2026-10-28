use serde::Deserialize;
use tracing::{debug, warn};

use crate::providers::llm::manager::LlmManager;
use crate::providers::llm::types::Message;

/// A single action chosen by the research agent on each step.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentAction {
    /// Run a web search with the given query.
    Search { query: String },
    /// Fetch a web page (converted to markdown) at the given URL.
    Fetch { url: String },
    /// Search for images.
    ImageSearch { query: String },
    /// Look at one image (models that see images).
    ViewImage { url: String },
    /// Search for videos: on every site through web search, or on one site with yt-dlp.
    VideoSearch { query: String, site: Option<String> },
    /// Read a video's details and subtitles: a part from `from` seconds, or the whole text.
    Video { url: String, from: Option<u64>, full: bool, lang: Option<String> },
    /// Read an attached file: a page, sheet rows, a section, the fragments matching a query, or
    /// its beginning.
    ReadFile { file: String, place: crate::attachments::Locator, query: Option<String> },
    /// Record an internal reasoning step.
    Think { thought: String },
    /// Produce the final markdown answer and finish, with up to three suggested follow-up questions.
    Answer { markdown: String, follow_ups: Vec<String> },
}

/// An earlier question of the conversation, given to the agent as context.
#[derive(Debug, Clone, PartialEq)]
pub struct PriorTurn {
    pub question: String,
    pub answer: String,
    /// Pages read for that turn: (url, title).
    pub sources: Vec<(String, String)>,
}

/// Default upper bound for the conversation history passed to the agent, in characters.
pub const CONTEXT_CHAR_LIMIT: usize = 12_000;
const MAX_FOLLOW_UPS: usize = 3;
/// The ```chart example of the prompt (a valid chart, checked by a test).
const CHART_EXAMPLE: &str = r#"{"type": "line", "title": "Smartphone sales", "x": ["2022", "2023", "2024"], "series": [{"name": "Apple", "data": [226, 235, 232]}, {"name": "Samsung", "data": [262, 226, 223]}], "unit": "M units"}"#;

/// The research agent decides the next tool call given the conversation so far.
pub struct ResearchAgent;

/// Which tools the agent may use in a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tools {
    /// Web search and page fetching.
    pub web: bool,
    /// Reading attached files.
    pub files: bool,
    /// Image search, and images in the answer.
    pub images: bool,
    /// Looking at images: the model sees them.
    pub view: bool,
    /// Video search and reading videos.
    pub videos: bool,
}

impl Default for Tools {
    fn default() -> Self {
        Self { web: true, files: false, images: false, view: false, videos: false }
    }
}

impl ResearchAgent {
    /// System prompt for web research without attached files.
    pub fn system_prompt(max_steps: usize) -> String {
        Self::system_prompt_with(max_steps, Tools::default())
    }

    /// System prompt describing the agent goal, the tools it may use and the
    /// strict JSON output contract used for every step.
    pub fn system_prompt_with(max_steps: usize, tools: Tools) -> String {
        let mut list = Vec::new();
        if tools.web {
            list.push(r#"search    — run a web search.            JSON: {"action": "search", "query": "<search query>"}"#.to_string());
            list.push(r#"fetch     — read a web page as markdown. JSON: {"action": "fetch", "url": "<absolute http(s) url>"}"#.to_string());
        }
        if tools.images {
            list.push(r#"image_search — search for images.       JSON: {"action": "image_search", "query": "<search query>"}"#.to_string());
        }
        if tools.images && tools.view {
            list.push(r#"view_image — look at one image.         JSON: {"action": "view_image", "url": "<image address>"}"#.to_string());
        }
        if tools.videos {
            list.push(
                r#"video_search — search for videos.      JSON: {"action": "video_search", "query": "<search query>"}
             Finds videos on every site; add "site" (youtube, bilibili, niconico, soundcloud) to search that site itself."#
                    .to_string(),
            );
            list.push(
                r#"video     — read a video's details and subtitles. JSON: {"action": "video", "url": "<video address>"}
             The text comes a part at a time with [m:ss] marks: add "from": "40:00" for a later part, "full": true
             for the whole text at once, or "lang": "en" for subtitles in another language."#
                    .to_string(),
            );
        }
        if tools.files {
            list.push(
                r#"read_file — read an attached file.      JSON: {"action": "read_file", "file": "F1", "page": 3}
             Instead of "page" give "sheet" and "rows": "40-60" for spreadsheets, "section" for a heading,
             "query" for the passages matching some words, or nothing for the beginning of the file."#
                    .to_string(),
            );
        }
        list.push(r#"think     — record private reasoning.     JSON: {"action": "think", "thought": "<your reasoning>"}"#.to_string());
        list.push(r#"answer    — finish with the answer.       JSON: {"action": "answer", "markdown": "<final answer in Markdown>"}"#.to_string());
        let tool_list = list.iter().enumerate().map(|(i, t)| format!("{}. {t}", i + 1)).collect::<Vec<_>>().join("\n");

        let (role, goal) = match (tools.web, tools.files) {
            (true, false) => ("web research agent", "by searching the web, reading pages, and reasoning"),
            (true, true) => ("research agent", "by reading the attached files, searching the web, reading pages, and reasoning"),
            _ => ("document research agent", "using only the attached files and reasoning; web search is turned off"),
        };
        let mut rules = vec![
            if tools.web { "Always begin by planning with a search or a think step." } else { "Always begin by planning with a think step or by reading a file." }.to_string(),
            "Use \"think\" steps to briefly explain your reasoning and plan between other steps, so your progress stays transparent to the user.".to_string(),
        ];
        if tools.web {
            rules.push("Only fetch URLs that appeared in earlier search results.".into());
            rules.push("Use multiple searches and fetches to gather enough evidence before answering.".into());
            rules.push("Cite sources in the final answer as Markdown links where appropriate.".into());
        }
        if tools.files {
            rules.push("The attached files are part of the request: read the parts you need before answering, and prefer them over the web for what they cover.".into());
            rules.push("Cite file passages as Markdown links to their attachment addresses with the place, e.g. [report.pdf, p. 3](attachment://ID?page=3).".into());
        }
        if tools.images {
            rules.push("Add images to the answer only when they help (what something looks like, diagrams, maps, products) or the user asks for them. Insert them as ![short caption](image address) next to the text they illustrate, a few at most. Use only addresses from image_search results or images (![alt](address)) on pages you fetched; never make addresses up.".into());
            if tools.view {
                rules.push("Look at an image with view_image before using it when its caption does not show that it fits.".into());
            } else {
                rules.push("You cannot see images: judge them by their captions, sites and the text around them on the page.".into());
            }
        }
        if tools.videos {
            rules.push("Use videos when they help (talks, lectures, tutorials, reviews, interviews) or the user asks for them. Read a video with \"video\" before relying on it; subtitles made by speech recognition or machine translation may contain errors.".into());
            rules.push("Cite a moment of a video as a link with its time from the [m:ss] marks, in the form the video result shows, e.g. [12:34](https://www.youtube.com/watch?v=ID&t=754s).".into());
            rules.push("Show a video in the answer as a card on a line of its own: [![Title](thumbnail address)](video address), with the thumbnail address from video_search or video results. Never make addresses up.".into());
        }
        if !tools.web {
            rules.push("Answer from only the attached files. If they do not contain the answer, say so.".into());
        }
        rules.push(format!(
            r#"Draw diagrams and charts only when a picture says more than text: processes, structures, timelines, comparisons of numbers, trends.
  Diagrams: a ```mermaid block (flowchart, sequenceDiagram, timeline, mindmap, classDiagram, stateDiagram-v2, erDiagram, gantt, quadrantChart). Keep labels short and quote those with spaces or punctuation: A["Step one"]. No styles, classDef, click or HTML.
  Charts: a ```chart block with one JSON object: "type" (one of {types}), "title", "x" (labels), "series" ([{{"name": ..., "data": [...]}}], one number per label, null for a gap), optional "unit", "x_title", "y_title", "stacked", "horizontal" (bars). Bars with a line: give the line series "type": "line" and "axis": "right", with "unit_right". pie and donut: one series of non-negative numbers. scatter: "data" of [x, y] pairs, no "x". radar: at least 3 labels. heatmap: "y" with row labels and one series whose "data" is rows of numbers. Use numbers from your sources. Example:
  ```chart
  {CHART_EXAMPLE}
  ```
  These blocks are inside the JSON string too: escape their quotes as \" and line breaks as \n."#,
            types = super::chart::TYPES.join(", ")
        ));
        let rules = rules.iter().map(|r| format!("- {r}")).collect::<Vec<_>>().join("\n");
        format!(
            r#"You are an autonomous {role}. Your goal is to answer the user's
request thoroughly and accurately {goal}.

You operate in a loop. On EACH turn you must call exactly ONE tool by replying with a
single JSON object and nothing else (no markdown fences, no commentary).

Available tools:
{tool_list}

Rules:
{rules}
- You have at most {max_steps} steps. When you have enough information, call "answer".
- The "answer" markdown must directly and completely address the user's request.
- Write mathematical formulas in LaTeX: \(...\) inside a sentence, and $$...$$ on lines
  of their own for displayed formulas. Inside the JSON string every backslash is doubled
  ("\\(E = mc^2\\)", "\\frac{{a}}{{b}}").
- Do not end the answer with offers of further help ("If you want, I can also…");
  instead put 2-3 short follow-up questions the user may ask next, written in the user's
  language, in the "follow_ups" field: {{"action": "answer", "markdown": "...", "follow_ups": ["...", "..."]}}
- If earlier questions of the conversation are given, answer the new request in their
  context. You may reuse their sources, and search or read pages again whenever needed.
- Respond with ONLY the JSON object for the chosen tool. Do not wrap it in code fences."#
        )
    }

    /// Ask the LLM for the next action given the running transcript.
    pub async fn decide_next_step(
        llm: &LlmManager,
        messages: Vec<Message>,
    ) -> Result<(AgentAction, u32, u32), String> {
        let response = llm
            .complete_for_stage("research", messages, true)
            .await
            .map_err(|e| format!("LLM error: {e}"))?;

        let action = Self::parse_action(&response.content).map_err(|_| {
            let message = "The model response did not contain a valid research action".to_string();
            llm.report_invalid_response("research", &message, &response, crate::providers::llm::IssueOutcome::Continued);
            message
        })?;
        Ok((action, response.prompt_tokens, response.completion_tokens))
    }

    /// Parse a model reply into an [`AgentAction`].
    ///
    /// Tolerates code fences and surrounding text by extracting the first
    /// JSON object found in the reply. If the JSON object is truncated
    /// (e.g. a long answer cut off by the token limit), it falls back to a
    /// lenient field-extraction salvage so partial answers are not lost.
    pub fn parse_action(raw: &str) -> Result<AgentAction, String> {
        // Strict path: a complete, well-formed JSON object.
        if let Some(json_str) = extract_json_object(raw) {
            match parse_json_action(&json_str) {
                Ok(action) => return Ok(action),
                Err(strict_err) => {
                    if let Some(action) = salvage_action(raw) {
                        return Ok(action);
                    }
                    return Err(strict_err);
                }
            }
        }

        // No complete JSON object (likely truncated) — attempt salvage.
        salvage_action(raw)
            .ok_or_else(|| format!("No JSON object found in model reply: {raw}"))
    }

    /// Earlier turns as one context message, or `None` for the first question.
    /// Page text is not repeated; when the history is too long, older answers are shortened first.
    pub fn conversation_context(turns: &[PriorTurn], limit: usize) -> Option<String> {
        if turns.is_empty() {
            return None;
        }
        let render = |answer_limits: &[usize]| {
            let mut out = String::from("Earlier in this conversation:\n");
            for (i, turn) in turns.iter().enumerate() {
                let limit = answer_limits[i];
                let answer = if turn.answer.chars().count() > limit {
                    format!("{} [answer shortened]", turn.answer.chars().take(limit).collect::<String>())
                } else {
                    turn.answer.clone()
                };
                out.push_str(&format!("\nQuestion {}: {}\nAnswer {}:\n{}\n", i + 1, turn.question, i + 1, answer));
                if !turn.sources.is_empty() {
                    out.push_str("Sources read:\n");
                    for (url, title) in &turn.sources {
                        out.push_str(&format!("- {title} — {url}\n"));
                    }
                }
            }
            out
        };
        let mut limits: Vec<usize> = turns.iter().map(|t| t.answer.chars().count()).collect();
        let mut text = render(&limits);
        // Shorten the oldest answers first, keeping at least a short summary of each.
        for i in 0..turns.len() {
            if text.chars().count() <= limit {
                break;
            }
            let excess = text.chars().count() - limit;
            limits[i] = limits[i].saturating_sub(excess).max(400.min(limits[i]));
            text = render(&limits);
        }
        Some(text)
    }

    /// The `<title>` of an HTML page, cleaned up and capped at 160 characters.
    pub fn page_title(html: &str) -> Option<String> {
        let lower = html.to_lowercase();
        let start = lower.find("<title")?;
        let open_end = lower[start..].find('>')? + start + 1;
        let close = lower[open_end..].find("</title>")? + open_end;
        let raw = &html[open_end..close];
        let decoded = raw
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");
        let title = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
        if title.is_empty() {
            None
        } else {
            Some(title.chars().take(160).collect())
        }
    }

    /// Convert fetched HTML into Markdown. Falls back to plain text extraction
    /// when conversion fails or produces nothing useful.
    pub fn html_to_markdown(html: &str, url: &str) -> String {
        let html = absolute_images(html, url);
        match htmd::convert(&html) {
            Ok(md) if !md.trim().is_empty() => md,
            _ => {
                warn!(url = %url, "htmd conversion failed/empty, falling back to text extraction");
                let doc = crate::roles::document_parser::DocumentParser::parse(&html, url);
                doc.text
            }
        }
    }
}

/// Images in Markdown, in order: (address, alt text), each address once.
pub fn markdown_images(markdown: &str) -> Vec<(String, String)> {
    static IMAGE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r#"!\[([^\]]*)\]\(\s*<?([^\s)>]+)>?(?:\s+"[^"]*")?\s*\)"#).unwrap());
    let mut found: Vec<(String, String)> = Vec::new();
    for caps in IMAGE.captures_iter(markdown) {
        let url = caps[2].to_string();
        if !found.iter().any(|(u, _)| *u == url) {
            found.push((url, caps[1].trim().to_string()));
        }
    }
    found
}

/// Video cards in Markdown, in order: `[![title](thumbnail)](video)` as (title, thumbnail, video),
/// each thumbnail once.
pub fn video_cards(markdown: &str) -> Vec<(String, String, String)> {
    static CARD: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r#"\[\s*!\[([^\]]*)\]\(\s*<?([^\s)>]+)>?(?:\s+"[^"]*")?\s*\)\s*\]\(\s*<?([^\s)>]+)>?(?:\s+"[^"]*")?\s*\)"#).unwrap()
    });
    let mut found: Vec<(String, String, String)> = Vec::new();
    for caps in CARD.captures_iter(markdown) {
        if !found.iter().any(|(_, thumbnail, _)| *thumbnail == caps[2]) {
            found.push((caps[1].trim().to_string(), caps[2].to_string(), caps[3].to_string()));
        }
    }
    found
}

/// Page images with addresses the agent can use: relative and lazy-loaded sources become
/// absolute `http(s)` addresses; inline `data:` images and tracking pixels are dropped.
fn absolute_images(html: &str, page_url: &str) -> String {
    static IMG: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| regex::Regex::new(r"(?is)<img\b[^>]*>").unwrap());
    static ATTR: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r#"(?is)([a-z][a-z0-9_:-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+))"#).unwrap()
    });
    let Ok(base) = url::Url::parse(page_url) else { return html.to_string() };
    IMG.replace_all(html, |caps: &regex::Captures| {
        let tag = &caps[0];
        let attrs: Vec<(String, String)> = ATTR
            .captures_iter(tag)
            .map(|a| {
                let value = a.get(2).or(a.get(3)).or(a.get(4)).map_or("", |m| m.as_str());
                (a[1].to_lowercase(), value.replace("&amp;", "&").trim().to_string())
            })
            .collect();
        let get = |name: &str| attrs.iter().find(|(n, v)| n == name && !v.is_empty()).map(|(_, v)| v.as_str());
        let tiny = ["width", "height"].iter().any(|n| get(n).and_then(|v| v.trim_end_matches("px").parse::<u32>().ok()).is_some_and(|v| v <= 2));
        if tiny {
            return String::new();
        }
        // The largest candidate of a srcset ("a.jpg 480w, b.jpg 1080w").
        let from_srcset = |name: &str| {
            get(name).and_then(|set| {
                set.split(',')
                    .filter_map(|c| {
                        let mut parts = c.split_whitespace();
                        let url = parts.next()?;
                        let size = parts.next().and_then(|d| d.trim_end_matches(['w', 'x']).parse::<f64>().ok()).unwrap_or(1.0);
                        Some((url, size))
                    })
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(url, _)| url)
            })
        };
        let usable = |v: &&str| !v.starts_with("data:") && !v.starts_with("blob:");
        let source = get("src")
            .filter(usable)
            .or_else(|| ["data-src", "data-lazy-src", "data-original", "data-url"].into_iter().find_map(|n| get(n).filter(usable)))
            .or_else(|| from_srcset("srcset").filter(usable))
            .or_else(|| from_srcset("data-srcset").filter(usable));
        let Some(address) = source.and_then(|s| base.join(s).ok()).filter(|u| matches!(u.scheme(), "http" | "https")) else {
            return String::new();
        };
        let escape = |v: &str| v.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;");
        format!(r#"<img src="{}" alt="{}">"#, escape(address.as_str()), escape(get("alt").unwrap_or("")))
    })
    .into_owned()
}

/// Extract the first balanced JSON object substring from a string.
fn extract_json_object(raw: &str) -> Option<String> {
    let start = raw.find('{')?;
    let bytes = raw.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (i, &b) in bytes.iter().enumerate().skip(start) {
        let c = b as char;
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let candidate = &raw[start..=i];
                    debug!(len = candidate.len(), "Extracted JSON object from model reply");
                    return Some(candidate.to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse a complete JSON object string into an [`AgentAction`].
fn parse_json_action(json_str: &str) -> Result<AgentAction, String> {
    #[derive(Deserialize)]
    struct RawAction {
        action: String,
        #[serde(default)]
        query: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        thought: Option<String>,
        #[serde(default)]
        file: Option<String>,
        #[serde(default)]
        page: Option<serde_json::Value>,
        #[serde(default)]
        sheet: Option<String>,
        #[serde(default)]
        rows: Option<serde_json::Value>,
        #[serde(default)]
        section: Option<String>,
        #[serde(default)]
        markdown: Option<String>,
        #[serde(default)]
        follow_ups: Option<Vec<String>>,
        #[serde(default)]
        site: Option<String>,
        #[serde(default)]
        from: Option<serde_json::Value>,
        #[serde(default)]
        full: Option<serde_json::Value>,
        #[serde(default)]
        lang: Option<String>,
    }

    let parsed: RawAction = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse agent action JSON: {e} (raw: {json_str})"))?;

    match parsed.action.trim().to_lowercase().as_str() {
        "search" => {
            let query = parsed
                .query
                .filter(|q| !q.trim().is_empty())
                .ok_or_else(|| "search action missing 'query'".to_string())?;
            Ok(AgentAction::Search { query })
        }
        "fetch" => {
            let url = parsed
                .url
                .filter(|u| !u.trim().is_empty())
                .ok_or_else(|| "fetch action missing 'url'".to_string())?;
            Ok(AgentAction::Fetch { url })
        }
        "image_search" => {
            let query = parsed
                .query
                .filter(|q| !q.trim().is_empty())
                .ok_or_else(|| "image_search action missing 'query'".to_string())?;
            Ok(AgentAction::ImageSearch { query })
        }
        "view_image" => {
            let url = parsed
                .url
                .filter(|u| !u.trim().is_empty())
                .ok_or_else(|| "view_image action missing 'url'".to_string())?;
            Ok(AgentAction::ViewImage { url: url.trim().to_string() })
        }
        "video_search" => {
            let query = parsed
                .query
                .filter(|q| !q.trim().is_empty())
                .ok_or_else(|| "video_search action missing 'query'".to_string())?;
            let site = parsed.site.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            Ok(AgentAction::VideoSearch { query, site })
        }
        "video" | "watch_video" | "read_video" => {
            let url = parsed
                .url
                .filter(|u| !u.trim().is_empty())
                .ok_or_else(|| "video action missing 'url'".to_string())?;
            let from = parsed.from.as_ref().and_then(|value| match value {
                serde_json::Value::Number(n) => n.as_u64(),
                serde_json::Value::String(text) => crate::providers::video::subtitles::parse_clock(text),
                _ => None,
            });
            let full = parsed.full.as_ref().is_some_and(|value| value.as_bool().unwrap_or_else(|| value.as_str() == Some("true")));
            let lang = parsed.lang.map(|l| l.trim().to_string()).filter(|l| !l.is_empty());
            Ok(AgentAction::Video { url: url.trim().to_string(), from, full, lang })
        }
        "read_file" | "read" => {
            let file = parsed
                .file
                .or(parsed.url)
                .filter(|f| !f.trim().is_empty())
                .ok_or_else(|| "read_file action missing 'file'".to_string())?;
            let number = |value: &serde_json::Value| value.as_u64().or_else(|| value.as_str()?.trim().parse().ok()).map(|n| n as u32);
            let rows = parsed.rows.as_ref().and_then(|value| match value {
                serde_json::Value::Array(pair) if pair.len() == 2 => Some((number(&pair[0])?, number(&pair[1])?)),
                serde_json::Value::String(text) => {
                    let (a, b) = text.split_once(['-', '–']).unwrap_or((text, text));
                    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
                }
                other => number(other).map(|n| (n, n)),
            });
            let place = crate::attachments::Locator {
                page: parsed.page.as_ref().and_then(number),
                sheet: parsed.sheet.filter(|s| !s.trim().is_empty()),
                rows,
                section: parsed.section.filter(|s| !s.trim().is_empty()),
            };
            let query = parsed.query.filter(|q| !q.trim().is_empty());
            Ok(AgentAction::ReadFile { file, place, query })
        }
        "think" => {
            let thought = parsed.thought.unwrap_or_default();
            Ok(AgentAction::Think { thought })
        }
        "answer" => {
            let markdown = parsed
                .markdown
                .filter(|m| !m.trim().is_empty())
                .ok_or_else(|| "answer action missing 'markdown'".to_string())?;
            let follow_ups = parsed
                .follow_ups
                .unwrap_or_default()
                .into_iter()
                .map(|q| q.trim().to_string())
                .filter(|q| !q.is_empty())
                .take(MAX_FOLLOW_UPS)
                .collect();
            Ok(AgentAction::Answer { markdown, follow_ups })
        }
        other => Err(format!("Unknown agent action: {other}")),
    }
}

/// Best-effort recovery from a possibly-truncated JSON reply.
///
/// Long answers can exceed the model's token limit, producing a JSON object
/// whose final string value is cut off mid-content. This extracts the action
/// type and the relevant field directly, tolerating a missing closing quote.
fn salvage_action(raw: &str) -> Option<AgentAction> {
    let action_type = extract_string_field(raw, "action")?.trim().to_lowercase();
    match action_type.as_str() {
        "answer" => {
            let markdown = extract_string_field(raw, "markdown")?;
            if markdown.trim().is_empty() {
                None
            } else {
                debug!(len = markdown.len(), "Salvaged truncated answer");
                Some(AgentAction::Answer { markdown, follow_ups: vec![] })
            }
        }
        "think" => Some(AgentAction::Think {
            thought: extract_string_field(raw, "thought").unwrap_or_default(),
        }),
        "search" => {
            let query = extract_string_field(raw, "query")?;
            (!query.trim().is_empty()).then_some(AgentAction::Search { query })
        }
        "fetch" => {
            let url = extract_string_field(raw, "url")?;
            (!url.trim().is_empty()).then_some(AgentAction::Fetch { url })
        }
        _ => None,
    }
}

/// Extract the string value of a JSON field by name, tolerating truncation.
///
/// Reads from the opening quote up to the first unescaped closing quote, or to
/// the end of input if the value was cut off. The captured content is
/// JSON-unescaped (strictly when possible, leniently otherwise).
fn extract_string_field(raw: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\"");
    let key_pos = raw.find(&key)?;
    let after_key = &raw[key_pos + key.len()..];
    let colon = after_key.find(':')?;
    let after_colon = &after_key[colon + 1..];
    let quote = after_colon.find('"')?;
    let content = &after_colon[quote + 1..];

    let bytes = content.as_bytes();
    let mut escaped = false;
    let mut end = content.len();
    for (i, &b) in bytes.iter().enumerate() {
        let c = b as char;
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            end = i;
            break;
        }
    }
    let captured = &content[..end];

    // Strict unescape first (drop a dangling backslash from truncation).
    let trimmed = captured.trim_end_matches('\\');
    if let Ok(s) = serde_json::from_str::<String>(&format!("\"{trimmed}\"")) {
        return Some(s);
    }
    Some(lenient_unescape(captured))
}

/// Minimal JSON string unescaping for salvaged (possibly invalid) content.
fn lenient_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => {} // trailing backslash from truncation — drop it
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_carry_up_to_three_follow_up_suggestions() {
        let raw = r#"{"action":"answer","markdown":"Done","follow_ups":["Cheapest?", "  ", "Mobile only?", "Free lists?", "Fourth"]}"#;
        assert_eq!(
            ResearchAgent::parse_action(raw).unwrap(),
            AgentAction::Answer {
                markdown: "Done".to_string(),
                follow_ups: vec!["Cheapest?".into(), "Mobile only?".into(), "Free lists?".into()],
            }
        );
        let plain = r#"{"action":"answer","markdown":"Done"}"#;
        assert_eq!(
            ResearchAgent::parse_action(plain).unwrap(),
            AgentAction::Answer { markdown: "Done".to_string(), follow_ups: vec![] }
        );
    }

    #[test]
    fn the_prompt_asks_for_suggestions_instead_of_offers_in_the_answer() {
        let prompt = ResearchAgent::system_prompt(16);
        assert!(prompt.contains("\"follow_ups\""));
        assert!(prompt.contains("at most 16 steps"));
        assert!(prompt.contains("Do not end the answer with offers"));
    }

    #[test]
    fn the_prompt_asks_for_formulas_in_latex() {
        let prompt = ResearchAgent::system_prompt(16);
        assert!(prompt.contains(r"formulas in LaTeX: \(...\) inside a sentence"));
        assert!(prompt.contains("$$...$$ on lines"));
        assert!(prompt.contains(r#""\\frac{a}{b}""#));
    }

    #[test]
    fn the_prompt_describes_diagrams_and_charts() {
        for tools in [Tools { web: true, files: false, images: true, view: true, videos: true }, Tools { web: false, files: true, images: false, view: false, videos: false }] {
            let prompt = ResearchAgent::system_prompt_with(16, tools);
            assert!(prompt.contains("```mermaid"), "{prompt}");
            assert!(prompt.contains("```chart"));
            assert!(prompt.contains(&crate::roles::chart::TYPES.join(", ")));
            assert!(prompt.contains(CHART_EXAMPLE));
            assert!(prompt.contains("only when"));
            assert!(prompt.contains(r#"escape their quotes as \" and line breaks as \n"#), "the blocks sit inside a JSON string");
        }
        // The example the model copies is a valid chart.
        crate::roles::chart::check(CHART_EXAMPLE).unwrap();
    }

    #[test]
    fn earlier_turns_become_context_without_page_text() {
        assert_eq!(ResearchAgent::conversation_context(&[], CONTEXT_CHAR_LIMIT), None);
        let turns = vec![
            PriorTurn {
                question: "Find Malaysian proxies".into(),
                answer: "Use Proxy-Seller.".into(),
                sources: vec![("https://proxy-seller.me/my".into(), "Proxy-Seller".into())],
            },
            PriorTurn { question: "Which are cheapest?".into(), answer: "Proxy5.".into(), sources: vec![] },
        ];
        let context = ResearchAgent::conversation_context(&turns, CONTEXT_CHAR_LIMIT).unwrap();
        assert!(context.contains("Question 1: Find Malaysian proxies"));
        assert!(context.contains("Answer 1:\nUse Proxy-Seller."));
        assert!(context.contains("- Proxy-Seller — https://proxy-seller.me/my"));
        assert!(context.contains("Question 2: Which are cheapest?"));
    }

    #[test]
    fn long_history_shortens_older_answers_first() {
        let long = "x".repeat(20_000);
        let turns = vec![
            PriorTurn { question: "Old".into(), answer: long.clone(), sources: vec![] },
            PriorTurn { question: "Recent".into(), answer: "Short recent answer".into(), sources: vec![] },
        ];
        let context = ResearchAgent::conversation_context(&turns, CONTEXT_CHAR_LIMIT).unwrap();
        assert!(context.chars().count() < CONTEXT_CHAR_LIMIT + 500);
        assert!(context.contains("[answer shortened]"));
        assert!(context.contains("Short recent answer"));
    }

    #[test]
    fn page_titles_come_from_the_html_title() {
        assert_eq!(
            ResearchAgent::page_title("<html><head><title>\n  Malaysia &amp; proxies  | Proxy-Seller\n</title></head></html>"),
            Some("Malaysia & proxies | Proxy-Seller".to_string())
        );
        assert_eq!(ResearchAgent::page_title("<html><body>No title</body></html>"), None);
        assert_eq!(ResearchAgent::page_title(&format!("<title>{}</title>", "a".repeat(400))).unwrap().chars().count(), 160);
    }

    #[test]
    fn test_parse_search_action() {
        let raw = r#"{"action": "search", "query": "rust async runtime"}"#;
        let action = ResearchAgent::parse_action(raw).unwrap();
        assert_eq!(
            action,
            AgentAction::Search {
                query: "rust async runtime".to_string()
            }
        );
    }

    #[test]
    fn test_parse_fetch_action() {
        let raw = r#"{"action":"fetch","url":"https://example.com/post"}"#;
        let action = ResearchAgent::parse_action(raw).unwrap();
        assert_eq!(
            action,
            AgentAction::Fetch {
                url: "https://example.com/post".to_string()
            }
        );
    }

    #[test]
    fn read_file_actions_name_a_place_or_a_query() {
        use crate::attachments::Locator;
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"read_file","file":"F1","page":"3"}"#).unwrap(),
            AgentAction::ReadFile { file: "F1".into(), place: Locator::page(3), query: None }
        );
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"read_file","file":"prices.xlsx","sheet":"Data","rows":"40-60"}"#).unwrap(),
            AgentAction::ReadFile {
                file: "prices.xlsx".into(),
                place: Locator { sheet: Some("Data".into()), rows: Some((40, 60)), ..Locator::default() },
                query: None
            }
        );
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"read_file","file":"F2","rows":[5,9],"query":"revenue"}"#).unwrap(),
            AgentAction::ReadFile { file: "F2".into(), place: Locator { rows: Some((5, 9)), ..Locator::default() }, query: Some("revenue".into()) }
        );
        assert!(ResearchAgent::parse_action(r#"{"action":"read_file"}"#).is_err());
    }

    #[test]
    fn the_prompt_offers_only_the_tools_of_the_run() {
        let web = ResearchAgent::system_prompt(10);
        assert!(web.contains(r#""action": "search""#) && !web.contains("read_file"));
        let both = ResearchAgent::system_prompt_with(10, Tools { web: true, files: true, ..Default::default() });
        assert!(both.contains(r#""action": "search""#) && both.contains("read_file") && both.contains("attachment://ID?page=3"));
        let files = ResearchAgent::system_prompt_with(10, Tools { web: false, files: true, ..Default::default() });
        assert!(!files.contains(r#""action": "search""#) && !files.contains(r#""action": "fetch""#));
        assert!(files.contains("only the attached files") && files.contains("You have at most 10 steps"));
    }

    #[test]
    fn test_parse_think_action() {
        let raw = r#"{"action":"think","thought":"I should compare two sources"}"#;
        let action = ResearchAgent::parse_action(raw).unwrap();
        assert!(matches!(action, AgentAction::Think { .. }));
    }

    #[test]
    fn test_parse_answer_action() {
        let raw = r#"{"action":"answer","markdown":"Result heading and body"}"#;
        let action = ResearchAgent::parse_action(raw).unwrap();
        match action {
            AgentAction::Answer { markdown, .. } => assert!(markdown.contains("Result heading")),
            _ => panic!("expected answer"),
        }
    }

    #[test]
    fn test_parse_action_with_code_fence() {
        let raw = "Here you go:\n```json\n{\"action\": \"search\", \"query\": \"foo\"}\n```";
        let action = ResearchAgent::parse_action(raw).unwrap();
        assert_eq!(
            action,
            AgentAction::Search {
                query: "foo".to_string()
            }
        );
    }

    #[test]
    fn test_parse_action_with_nested_braces() {
        let raw = r#"{"action":"answer","markdown":"Use {curly} braces { nested }"}"#;
        let action = ResearchAgent::parse_action(raw).unwrap();
        match action {
            AgentAction::Answer { markdown, .. } => assert!(markdown.contains("{curly}")),
            _ => panic!("expected answer"),
        }
    }

    #[test]
    fn test_parse_action_missing_field() {
        let raw = r#"{"action":"search"}"#;
        assert!(ResearchAgent::parse_action(raw).is_err());
    }

    #[test]
    fn test_parse_action_unknown() {
        let raw = r#"{"action":"dance"}"#;
        assert!(ResearchAgent::parse_action(raw).is_err());
    }

    #[test]
    fn test_html_to_markdown_basic() {
        let html = "<html><body><h1>Title</h1><p>Hello <a href=\"https://x.com\">link</a></p></body></html>";
        let md = ResearchAgent::html_to_markdown(html, "https://example.com");
        assert!(md.contains("Title"));
        assert!(md.contains("Hello"));
    }

    #[test]
    fn image_actions_are_parsed() {
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"image_search","query":"red panda"}"#).unwrap(),
            AgentAction::ImageSearch { query: "red panda".into() }
        );
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"view_image","url":" https://x.com/a.jpg "}"#).unwrap(),
            AgentAction::ViewImage { url: "https://x.com/a.jpg".into() }
        );
        assert!(ResearchAgent::parse_action(r#"{"action":"image_search"}"#).is_err());
        assert!(ResearchAgent::parse_action(r#"{"action":"view_image","url":""}"#).is_err());
    }

    #[test]
    fn image_tools_are_offered_as_allowed() {
        let none = ResearchAgent::system_prompt(10);
        assert!(!none.contains("image_search") && !none.contains("view_image"));
        let blind = ResearchAgent::system_prompt_with(10, Tools { images: true, ..Default::default() });
        assert!(blind.contains(r#"{"action": "image_search""#));
        assert!(!blind.contains("view_image"));
        assert!(blind.contains("![short caption](image address)"));
        assert!(blind.contains("images (![alt](address)) on pages you fetched"));
        assert!(blind.contains("You cannot see images"));
        let seeing = ResearchAgent::system_prompt_with(10, Tools { images: true, view: true, ..Default::default() });
        assert!(seeing.contains(r#"{"action": "view_image""#));
        assert!(seeing.contains("Look at an image with view_image before using it"));
        // Looking at images comes only with image search.
        assert!(!ResearchAgent::system_prompt_with(10, Tools { view: true, ..Default::default() }).contains("view_image"));
    }

    #[test]
    fn page_images_get_absolute_addresses() {
        let html = r#"<html><body><p>Intro</p>
            <img src="/img/panda.jpg" alt="Red panda">
            <img src="data:image/gif;base64,R0lGOD" data-src="lazy/cat.png?a=1&amp;b=2" alt="Cat">
            <img srcset="small.webp 480w, large.webp 1080w" alt="Dog">
            <img src="https://track.example/p.gif" width="1" height="1">
            <img src="data:image/png;base64,AAA" alt="Inline">
            <img src="javascript:alert(1)" alt="Bad">
            </body></html>"#;
        let md = ResearchAgent::html_to_markdown(html, "https://zoo.example/animals/index.html");
        let images = markdown_images(&md);
        assert_eq!(
            images,
            vec![
                ("https://zoo.example/img/panda.jpg".to_string(), "Red panda".to_string()),
                ("https://zoo.example/animals/lazy/cat.png?a=1&b=2".to_string(), "Cat".to_string()),
                ("https://zoo.example/animals/large.webp".to_string(), "Dog".to_string()),
            ],
            "{md}"
        );
        assert!(!md.contains("track.example") && !md.contains("data:") && !md.contains("javascript"));
    }

    #[test]
    fn video_cards_are_found_once_in_order() {
        let md = "Watch [![A talk](https://i.ytimg.com/a.jpg)](https://youtu.be/a) and [ ![B](<https://i.ytimg.com/b.jpg> \"t\") ](https://youtu.be/b), again [![A](https://i.ytimg.com/a.jpg)](https://youtu.be/a). Not a card: ![C](https://x.com/c.jpg)";
        assert_eq!(
            video_cards(md),
            vec![
                ("A talk".to_string(), "https://i.ytimg.com/a.jpg".to_string(), "https://youtu.be/a".to_string()),
                ("B".to_string(), "https://i.ytimg.com/b.jpg".to_string(), "https://youtu.be/b".to_string()),
            ]
        );
    }

    #[test]
    fn markdown_images_are_found_once_in_order() {
        let md = r#"Look ![A panda](https://x.com/p.jpg "Panda") and [a link](https://x.com/page) ![](<https://x.com/q.png>) ![again](https://x.com/p.jpg)"#;
        assert_eq!(
            markdown_images(md),
            vec![("https://x.com/p.jpg".to_string(), "A panda".to_string()), ("https://x.com/q.png".to_string(), String::new())]
        );
    }

    #[test]
    fn test_salvage_truncated_answer() {
        // JSON answer cut off mid-markdown (no closing quote/brace).
        let raw = r##"{"action":"answer","markdown":"# Heading\n\nSome long content that was cut off mid-sen"##;
        let action = ResearchAgent::parse_action(raw).unwrap();
        match action {
            AgentAction::Answer { markdown, .. } => {
                assert!(markdown.contains("# Heading"));
                assert!(markdown.contains("cut off mid-sen"));
                assert!(markdown.contains('\n'));
            }
            _ => panic!("expected salvaged answer"),
        }
    }

    #[test]
    fn test_salvage_truncated_answer_trailing_backslash() {
        // Truncation right after an escape character.
        let raw = "{\"action\":\"answer\",\"markdown\":\"Line one\\nLine two\\";
        let action = ResearchAgent::parse_action(raw).unwrap();
        match action {
            AgentAction::Answer { markdown, .. } => {
                assert!(markdown.contains("Line one"));
                assert!(markdown.contains("Line two"));
            }
            _ => panic!("expected salvaged answer"),
        }
    }

    #[test]
    fn video_actions_are_read() {
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"video_search","query":"rust async"}"#).unwrap(),
            AgentAction::VideoSearch { query: "rust async".into(), site: None }
        );
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"video_search","query":"rust","site":" YouTube "}"#).unwrap(),
            AgentAction::VideoSearch { query: "rust".into(), site: Some("YouTube".into()) }
        );
        assert!(ResearchAgent::parse_action(r#"{"action":"video_search","query":" "}"#).is_err());

        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"video","url":"https://youtu.be/x"}"#).unwrap(),
            AgentAction::Video { url: "https://youtu.be/x".into(), from: None, full: false, lang: None }
        );
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"video","url":"https://youtu.be/x","from":"40:00","lang":"de"}"#).unwrap(),
            AgentAction::Video { url: "https://youtu.be/x".into(), from: Some(2400), full: false, lang: Some("de".into()) }
        );
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"video","url":"https://youtu.be/x","from":754,"full":true}"#).unwrap(),
            AgentAction::Video { url: "https://youtu.be/x".into(), from: Some(754), full: true, lang: None }
        );
        assert_eq!(
            ResearchAgent::parse_action(r#"{"action":"video","url":"https://youtu.be/x","full":"true","from":"soon"}"#).unwrap(),
            AgentAction::Video { url: "https://youtu.be/x".into(), from: None, full: true, lang: None }
        );
        assert!(ResearchAgent::parse_action(r#"{"action":"video"}"#).is_err());
    }

    #[test]
    fn video_tools_are_offered_when_the_run_has_them() {
        let without = ResearchAgent::system_prompt_with(10, Tools::default());
        assert!(!without.contains("video_search") && !without.contains(r#""action": "video""#));

        let with = ResearchAgent::system_prompt_with(10, Tools { videos: true, ..Default::default() });
        assert!(with.contains(r#"{"action": "video_search", "query": "<search query>"}"#));
        assert!(with.contains(r#""site""#) && with.contains("youtube, bilibili, niconico, soundcloud"));
        assert!(with.contains(r#"{"action": "video", "url": "<video address>"}"#));
        assert!(with.contains(r#""from": "40:00""#) && with.contains(r#""full": true"#) && with.contains(r#""lang""#));
        // Moments are cited with time links, and videos shown as cards.
        assert!(with.contains("[![Title](thumbnail address)](video address)"));
        assert!(with.contains("t=754"));
    }
}

