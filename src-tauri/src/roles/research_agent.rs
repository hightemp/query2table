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

/// Upper bound for the conversation history passed to the agent, in characters.
pub const CONTEXT_CHAR_LIMIT: usize = 12_000;
const MAX_FOLLOW_UPS: usize = 3;

/// The research agent decides the next tool call given the conversation so far.
pub struct ResearchAgent;

/// Which tools the agent may use in a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tools {
    /// Web search and page fetching.
    pub web: bool,
    /// Reading attached files.
    pub files: bool,
}

impl Default for Tools {
    fn default() -> Self {
        Self { web: true, files: false }
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
        if !tools.web {
            rules.push("Answer from only the attached files. If they do not contain the answer, say so.".into());
        }
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
    pub fn conversation_context(turns: &[PriorTurn]) -> Option<String> {
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
            if text.chars().count() <= CONTEXT_CHAR_LIMIT {
                break;
            }
            let excess = text.chars().count() - CONTEXT_CHAR_LIMIT;
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
        match htmd::convert(html) {
            Ok(md) if !md.trim().is_empty() => md,
            _ => {
                warn!(url = %url, "htmd conversion failed/empty, falling back to text extraction");
                let doc = crate::roles::document_parser::DocumentParser::parse(html, url);
                doc.text
            }
        }
    }
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
    fn earlier_turns_become_context_without_page_text() {
        assert_eq!(ResearchAgent::conversation_context(&[]), None);
        let turns = vec![
            PriorTurn {
                question: "Find Malaysian proxies".into(),
                answer: "Use Proxy-Seller.".into(),
                sources: vec![("https://proxy-seller.me/my".into(), "Proxy-Seller".into())],
            },
            PriorTurn { question: "Which are cheapest?".into(), answer: "Proxy5.".into(), sources: vec![] },
        ];
        let context = ResearchAgent::conversation_context(&turns).unwrap();
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
        let context = ResearchAgent::conversation_context(&turns).unwrap();
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
        let both = ResearchAgent::system_prompt_with(10, Tools { web: true, files: true });
        assert!(both.contains(r#""action": "search""#) && both.contains("read_file") && both.contains("attachment://ID?page=3"));
        let files = ResearchAgent::system_prompt_with(10, Tools { web: false, files: true });
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
}
