//! Interface language for the few texts the backend shows itself (notifications, tray menu).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ru,
}

/// The language for the `ui_language` setting ("system", "en" or "ru").
/// "system" follows `system_locale`, e.g. the value of LC_ALL or LANG ("ru_RU.UTF-8").
pub fn resolve_language(setting: Option<&str>, system_locale: Option<&str>) -> Lang {
    match setting {
        Some("ru") => Lang::Ru,
        Some("en") => Lang::En,
        _ if system_locale.is_some_and(|locale| locale.to_lowercase().starts_with("ru")) => Lang::Ru,
        _ => Lang::En,
    }
}

/// The language from saved settings and the environment.
pub fn current_language(settings: &std::collections::HashMap<String, String>) -> Lang {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()));
    resolve_language(settings.get("ui_language").map(String::as_str), locale.as_deref())
}

/// Title and body of the desktop notification for a run status, if it deserves one.
pub fn run_notification(status: &str, lang: Lang) -> Option<(&'static str, &'static str)> {
    match (status, lang) {
        ("completed", Lang::En) => Some(("Research Complete", "Your query has finished and results are ready.")),
        ("completed", Lang::Ru) => Some(("Прогон завершён", "Запрос выполнен, результаты готовы.")),
        ("failed", Lang::En) => Some(("Research Failed", "Your query encountered an error.")),
        ("failed", Lang::Ru) => Some(("Прогон завершился с ошибкой", "При выполнении запроса произошла ошибка.")),
        _ => None,
    }
}

/// Labels of the tray menu: (show window, quit).
pub fn tray_labels(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::En => ("Show Window", "Quit"),
        Lang::Ru => ("Показать окно", "Выход"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chosen_language_wins_and_system_follows_the_locale() {
        assert_eq!(resolve_language(Some("ru"), Some("en_US.UTF-8")), Lang::Ru);
        assert_eq!(resolve_language(Some("en"), Some("ru_RU.UTF-8")), Lang::En);
        assert_eq!(resolve_language(Some("system"), Some("ru_RU.UTF-8")), Lang::Ru);
        assert_eq!(resolve_language(None, Some("ru")), Lang::Ru);
        assert_eq!(resolve_language(Some("system"), Some("de_DE.UTF-8")), Lang::En);
        assert_eq!(resolve_language(Some("system"), None), Lang::En);
    }

    #[test]
    fn notifications_and_tray_speak_the_language() {
        assert_eq!(run_notification("completed", Lang::En).unwrap().0, "Research Complete");
        assert_eq!(run_notification("completed", Lang::Ru).unwrap().0, "Прогон завершён");
        assert_eq!(run_notification("failed", Lang::Ru).unwrap().0, "Прогон завершился с ошибкой");
        for status in ["cancelled", "running", "paused", "pending", "schema_review"] {
            assert!(run_notification(status, Lang::En).is_none(), "{status}");
        }
        assert_eq!(tray_labels(Lang::En), ("Show Window", "Quit"));
        assert_eq!(tray_labels(Lang::Ru), ("Показать окно", "Выход"));
    }
}
