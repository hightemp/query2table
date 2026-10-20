//! Services that speak the OpenAI Chat Completions API, offered as providers of their own.
//! They all run through `OpenAiCompatibleProvider` with the preset's address; the list is
//! mirrored in `src/lib/settings/llmProviders.ts` (checked by a test below).
//!
//! Settings of a preset: `<id>_api_key`, `<id>_model`, `<id>_json_mode`, and `<id>_url` for
//! local servers, whose address can change. Cloud presets have a fixed address.

#[derive(Debug, PartialEq, Eq)]
pub struct Preset {
    /// `llm_provider` value and settings prefix; also the provider name in accounting.
    pub id: &'static str,
    pub name: &'static str,
    /// Chat Completions base address, with `/v1` or its equivalent.
    pub url: &'static str,
    /// Runs on the user's machine: free, address editable, key optional.
    pub local: bool,
    /// Sends `response_format: json_object` unless turned off in Settings.
    pub json_mode: bool,
    /// Accepts the `reasoning_effort` parameter (others reject unknown fields).
    pub reasoning_effort: bool,
    /// Takes `max_completion_tokens` instead of the deprecated `max_tokens`.
    pub completion_tokens: bool,
}

pub const PRESETS: &[Preset] = &[
    Preset { id: "deepseek", name: "DeepSeek", url: "https://api.deepseek.com/v1", local: false, json_mode: true, reasoning_effort: false, completion_tokens: false },
    Preset { id: "groq", name: "Groq", url: "https://api.groq.com/openai/v1", local: false, json_mode: true, reasoning_effort: true, completion_tokens: false },
    Preset { id: "mistral", name: "Mistral", url: "https://api.mistral.ai/v1", local: false, json_mode: true, reasoning_effort: false, completion_tokens: false },
    Preset { id: "gemini", name: "Gemini", url: "https://generativelanguage.googleapis.com/v1beta/openai", local: false, json_mode: true, reasoning_effort: true, completion_tokens: false },
    Preset { id: "xai", name: "xAI", url: "https://api.x.ai/v1", local: false, json_mode: true, reasoning_effort: true, completion_tokens: false },
    Preset { id: "openai_cloud", name: "OpenAI", url: "https://api.openai.com/v1", local: false, json_mode: true, reasoning_effort: true, completion_tokens: true },
    // LM Studio documents only `json_schema` structured output.
    Preset { id: "lmstudio", name: "LM Studio", url: "http://localhost:1234/v1", local: true, json_mode: false, reasoning_effort: true, completion_tokens: false },
    Preset { id: "llamacpp", name: "llama.cpp", url: "http://localhost:8080/v1", local: true, json_mode: true, reasoning_effort: true, completion_tokens: false },
];

pub fn find(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.id == id)
}

impl Preset {
    /// OpenAI reasoning models (o-series, GPT-5) accept only the default temperature.
    pub fn fixed_temperature(&self, model: &str) -> bool {
        let model = model.trim().to_ascii_lowercase();
        self.id == "openai_cloud"
            && (model.starts_with("gpt-5")
                || (model.starts_with('o') && model[1..].starts_with(|c: char| c.is_ascii_digit())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_match_the_settings_page() {
        let source = include_str!("../../../../src/lib/settings/llmProviders.ts");
        let pattern = regex::Regex::new(
            r"\{ id: '([a-z_]+)', name: '([^']+)', url: '([^']+)', local: (true|false), jsonMode: (true|false)",
        )
        .unwrap();
        let listed: Vec<(String, String, String, bool, bool)> = pattern
            .captures_iter(source)
            .map(|c| (c[1].into(), c[2].into(), c[3].into(), &c[4] == "true", &c[5] == "true"))
            .collect();
        let expected: Vec<(String, String, String, bool, bool)> = PRESETS
            .iter()
            .map(|p| (p.id.into(), p.name.into(), p.url.into(), p.local, p.json_mode))
            .collect();
        assert_eq!(listed, expected);
    }

    #[test]
    fn preset_ids_do_not_clash_with_other_settings() {
        for preset in PRESETS {
            // `openai_*` belongs to "Custom address", `ollama_*` to Ollama.
            assert!(!["openai", "ollama", "ollama_cloud", "openrouter"].contains(&preset.id));
            assert!(!preset.url.ends_with('/'), "{}", preset.id);
        }
    }

    #[test]
    fn only_openai_reasoning_models_keep_the_default_temperature() {
        let openai = find("openai_cloud").unwrap();
        for model in ["gpt-5", "gpt-5-mini", "o3", "o4-mini", "o1"] {
            assert!(openai.fixed_temperature(model), "{model}");
        }
        for model in ["gpt-4.1-mini", "gpt-4o", "omni-moderation-latest"] {
            assert!(!openai.fixed_temperature(model), "{model}");
        }
        assert!(!find("groq").unwrap().fixed_temperature("o3"));
    }
}
