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
    /// Record an internal reasoning step.
    Think { thought: String },
    /// Produce the final markdown answer and finish.
    Answer { markdown: String },
}

/// The research agent decides the next tool call given the conversation so far.
pub struct ResearchAgent;

impl ResearchAgent {
    /// System prompt describing the agent goal, available tools and the
    /// strict JSON output contract used for every step.
    pub fn system_prompt(max_steps: usize) -> String {
        format!(
            r#"You are an autonomous web research agent. Your goal is to answer the user's
request thoroughly and accurately by searching the web, reading pages, and reasoning.

You operate in a loop. On EACH turn you must call exactly ONE tool by replying with a
single JSON object and nothing else (no markdown fences, no commentary).

Available tools:
1. search  — run a web search.        JSON: {{"action": "search", "query": "<search query>"}}
2. fetch   — read a web page as markdown. JSON: {{"action": "fetch", "url": "<absolute http(s) url>"}}
3. think   — record private reasoning.  JSON: {{"action": "think", "thought": "<your reasoning>"}}
4. answer  — finish with the answer.    JSON: {{"action": "answer", "markdown": "<final answer in Markdown>"}}

Rules:
- Always begin by planning with a search or a think step.
- Only fetch URLs that appeared in earlier search results.
- Use multiple searches and fetches to gather enough evidence before answering.
- Cite sources in the final answer as Markdown links where appropriate.
- You have at most {max_steps} steps. When you have enough information, call "answer".
- The "answer" markdown must directly and completely address the user's request.
- Respond with ONLY the JSON object for the chosen tool. Do not wrap it in code fences."#
        )
    }

    /// Ask the LLM for the next action given the running transcript.
    pub async fn decide_next_step(
        llm: &LlmManager,
        messages: Vec<Message>,
    ) -> Result<(AgentAction, u32, u32), String> {
        let response = llm
            .complete(messages, true)
            .await
            .map_err(|e| format!("LLM error: {e}"))?;

        let action = Self::parse_action(&response.content)?;
        Ok((action, response.prompt_tokens, response.completion_tokens))
    }

    /// Parse a model reply into an [`AgentAction`].
    ///
    /// Tolerates code fences and surrounding text by extracting the first
    /// JSON object found in the reply.
    pub fn parse_action(raw: &str) -> Result<AgentAction, String> {
        let json_str = extract_json_object(raw)
            .ok_or_else(|| format!("No JSON object found in model reply: {raw}"))?;

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
            markdown: Option<String>,
        }

        let parsed: RawAction = serde_json::from_str(&json_str)
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
            "think" => {
                let thought = parsed.thought.unwrap_or_default();
                Ok(AgentAction::Think { thought })
            }
            "answer" => {
                let markdown = parsed
                    .markdown
                    .filter(|m| !m.trim().is_empty())
                    .ok_or_else(|| "answer action missing 'markdown'".to_string())?;
                Ok(AgentAction::Answer { markdown })
            }
            other => Err(format!("Unknown agent action: {other}")),
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

#[cfg(test)]
mod tests {
    use super::*;

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
            AgentAction::Answer { markdown } => assert!(markdown.contains("Result heading")),
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
            AgentAction::Answer { markdown } => assert!(markdown.contains("{curly}")),
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
}
