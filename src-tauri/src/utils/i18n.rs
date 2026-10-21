//! Interface language for the few texts the backend shows itself (notifications, tray menu).
//! The languages match `src/lib/i18n/locales.ts`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ru,
    De,
    Fr,
    Es,
    It,
    Pt,
    Pl,
    Uk,
    Tr,
    Zh,
    Ja,
    Ko,
}

impl Lang {
    pub const ALL: [Lang; 13] = [
        Lang::En,
        Lang::Ru,
        Lang::De,
        Lang::Fr,
        Lang::Es,
        Lang::It,
        Lang::Pt,
        Lang::Pl,
        Lang::Uk,
        Lang::Tr,
        Lang::Zh,
        Lang::Ja,
        Lang::Ko,
    ];

    /// The `ui_language` value.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::Es => "es",
            Lang::It => "it",
            Lang::Pt => "pt",
            Lang::Pl => "pl",
            Lang::Uk => "uk",
            Lang::Tr => "tr",
            Lang::Zh => "zh",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
        }
    }

    fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|lang| lang.code() == code)
    }
}

/// The language for the `ui_language` setting ("system" or a language code).
/// "system" follows `system_locale`, e.g. the value of LC_ALL or LANG ("ru_RU.UTF-8"), by its
/// language part ("pt_PT" is Portuguese).
pub fn resolve_language(setting: Option<&str>, system_locale: Option<&str>) -> Lang {
    setting
        .and_then(Lang::from_code)
        .or_else(|| {
            let locale = system_locale?.to_lowercase();
            Lang::from_code(locale.split(['_', '-', '.', '@']).next()?)
        })
        .unwrap_or(Lang::En)
}

/// The language from saved settings and the environment.
pub fn current_language(settings: &std::collections::HashMap<String, String>) -> Lang {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()));
    resolve_language(settings.get("ui_language").map(String::as_str), locale.as_deref())
}

struct Texts {
    completed: (&'static str, &'static str),
    failed: (&'static str, &'static str),
    /// With `{level}`.
    budget_level: &'static str,
    budget_reached: &'static str,
    /// With `{spent}` and `{limit}`.
    budget_body: &'static str,
    /// Added after the body once the budget is reached.
    not_stopped: &'static str,
    tray: (&'static str, &'static str),
}

fn texts(lang: Lang) -> Texts {
    match lang {
        Lang::En => Texts {
            completed: ("Research Complete", "Your query has finished and results are ready."),
            failed: ("Research Failed", "Your query encountered an error."),
            budget_level: "Monthly budget at {level}%",
            budget_reached: "Monthly budget reached",
            budget_body: "Spent {spent} of {limit} this month.",
            not_stopped: " Runs are not stopped.",
            tray: ("Show Window", "Quit"),
        },
        Lang::Ru => Texts {
            completed: ("Прогон завершён", "Запрос выполнен, результаты готовы."),
            failed: ("Прогон завершился с ошибкой", "При выполнении запроса произошла ошибка."),
            budget_level: "Месячный бюджет: {level}%",
            budget_reached: "Месячный бюджет исчерпан",
            budget_body: "Потрачено {spent} из {limit} в этом месяце.",
            not_stopped: " Запуски не останавливаются.",
            tray: ("Показать окно", "Выход"),
        },
        Lang::De => Texts {
            completed: ("Abfrage abgeschlossen", "Ihre Abfrage ist fertig, die Ergebnisse liegen vor."),
            failed: ("Abfrage fehlgeschlagen", "Bei Ihrer Abfrage ist ein Fehler aufgetreten."),
            budget_level: "Monatsbudget zu {level} % ausgeschöpft",
            budget_reached: "Monatsbudget erreicht",
            budget_body: "Diesen Monat {spent} von {limit} ausgegeben.",
            not_stopped: " Ausführungen werden nicht angehalten.",
            tray: ("Fenster anzeigen", "Beenden"),
        },
        Lang::Fr => Texts {
            completed: ("Recherche terminée", "Votre requête est terminée, les résultats sont prêts."),
            failed: ("Échec de la recherche", "Votre requête a rencontré une erreur."),
            budget_level: "Budget mensuel à {level} %",
            budget_reached: "Budget mensuel atteint",
            budget_body: "{spent} dépensés sur {limit} ce mois-ci.",
            not_stopped: " Les exécutions ne sont pas arrêtées.",
            tray: ("Afficher la fenêtre", "Quitter"),
        },
        Lang::Es => Texts {
            completed: ("Búsqueda completada", "Tu consulta ha terminado y los resultados están listos."),
            failed: ("La búsqueda falló", "Se produjo un error en tu consulta."),
            budget_level: "Presupuesto mensual al {level} %",
            budget_reached: "Presupuesto mensual alcanzado",
            budget_body: "Gastado {spent} de {limit} este mes.",
            not_stopped: " Las ejecuciones no se detienen.",
            tray: ("Mostrar ventana", "Salir"),
        },
        Lang::It => Texts {
            completed: ("Ricerca completata", "La richiesta è terminata e i risultati sono pronti."),
            failed: ("Ricerca non riuscita", "Si è verificato un errore durante la richiesta."),
            budget_level: "Budget mensile al {level}%",
            budget_reached: "Budget mensile raggiunto",
            budget_body: "Spesi {spent} su {limit} questo mese.",
            not_stopped: " Le esecuzioni non vengono interrotte.",
            tray: ("Mostra finestra", "Esci"),
        },
        Lang::Pt => Texts {
            completed: ("Pesquisa concluída", "Sua consulta terminou e os resultados estão prontos."),
            failed: ("Falha na pesquisa", "Ocorreu um erro na sua consulta."),
            budget_level: "Orçamento mensal em {level}%",
            budget_reached: "Orçamento mensal atingido",
            budget_body: "Gasto de {spent} de {limit} neste mês.",
            not_stopped: " As execuções não são interrompidas.",
            tray: ("Mostrar janela", "Sair"),
        },
        Lang::Pl => Texts {
            completed: ("Wyszukiwanie zakończone", "Zapytanie zostało wykonane, wyniki są gotowe."),
            failed: ("Wyszukiwanie nie powiodło się", "Podczas wykonywania zapytania wystąpił błąd."),
            budget_level: "Budżet miesięczny: {level}%",
            budget_reached: "Budżet miesięczny wyczerpany",
            budget_body: "W tym miesiącu wydano {spent} z {limit}.",
            not_stopped: " Uruchomienia nie są zatrzymywane.",
            tray: ("Pokaż okno", "Zakończ"),
        },
        Lang::Uk => Texts {
            completed: ("Пошук завершено", "Запит виконано, результати готові."),
            failed: ("Помилка пошуку", "Під час виконання запиту сталася помилка."),
            budget_level: "Місячний бюджет: {level}%",
            budget_reached: "Місячний бюджет вичерпано",
            budget_body: "Цього місяця витрачено {spent} з {limit}.",
            not_stopped: " Запуски не зупиняються.",
            tray: ("Показати вікно", "Вийти"),
        },
        Lang::Tr => Texts {
            completed: ("Araştırma tamamlandı", "Sorgunuz tamamlandı, sonuçlar hazır."),
            failed: ("Araştırma başarısız oldu", "Sorgunuzda bir hata oluştu."),
            budget_level: "Aylık bütçe %{level}",
            budget_reached: "Aylık bütçe doldu",
            budget_body: "Bu ay {limit} bütçenin {spent} kadarı harcandı.",
            not_stopped: " Çalıştırmalar durdurulmaz.",
            tray: ("Pencereyi göster", "Çıkış"),
        },
        Lang::Zh => Texts {
            completed: ("研究完成", "查询已完成，结果已就绪。"),
            failed: ("研究失败", "查询时出错。"),
            budget_level: "月度预算已用 {level}%",
            budget_reached: "已达到月度预算",
            budget_body: "本月已花费 {spent}，预算为 {limit}。",
            not_stopped: "运行不会停止。",
            tray: ("显示窗口", "退出"),
        },
        Lang::Ja => Texts {
            completed: ("リサーチ完了", "クエリが完了し、結果の準備ができました。"),
            failed: ("リサーチ失敗", "クエリの実行中にエラーが発生しました。"),
            budget_level: "月間予算の {level}% を使用しました",
            budget_reached: "月間予算に達しました",
            budget_body: "今月の支出：{spent}（予算 {limit}）。",
            not_stopped: "実行は停止されません。",
            tray: ("ウィンドウを表示", "終了"),
        },
        Lang::Ko => Texts {
            completed: ("조사 완료", "쿼리가 끝났고 결과가 준비되었습니다."),
            failed: ("조사 실패", "쿼리 실행 중 오류가 발생했습니다."),
            budget_level: "월 예산 {level}% 사용",
            budget_reached: "월 예산 도달",
            budget_body: "이번 달 {limit} 중 {spent}를 사용했습니다.",
            not_stopped: " 실행은 중지되지 않습니다.",
            tray: ("창 표시", "종료"),
        },
    }
}

/// Title and body of the desktop notification for a run status, if it deserves one.
pub fn run_notification(status: &str, lang: Lang) -> Option<(&'static str, &'static str)> {
    match status {
        "completed" => Some(texts(lang).completed),
        "failed" => Some(texts(lang).failed),
        _ => None,
    }
}

/// The monthly budget warning at 80 or 100 percent.
pub fn budget_notification(level: u8, spent: f64, limit: f64, lang: Lang) -> (String, String) {
    let texts = texts(lang);
    let title = if level >= 100 { texts.budget_reached.to_string() } else { texts.budget_level.replace("{level}", &level.to_string()) };
    let mut body = texts.budget_body.replace("{spent}", &format!("${spent:.2}")).replace("{limit}", &format!("${limit:.2}"));
    if level >= 100 {
        body.push_str(texts.not_stopped);
    }
    (title, body)
}

/// Labels of the tray menu: (show window, quit).
pub fn tray_labels(lang: Lang) -> (&'static str, &'static str) {
    texts(lang).tray
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
        assert_eq!(resolve_language(Some("system"), Some("sv_SE.UTF-8")), Lang::En);
        assert_eq!(resolve_language(Some("system"), None), Lang::En);
    }

    #[test]
    fn every_interface_language_is_known_to_the_backend() {
        assert_eq!(Lang::ALL.len(), 13);
        for (code, locale, lang) in [
            ("de", "de_DE.UTF-8", Lang::De),
            ("fr", "fr_CA.UTF-8", Lang::Fr),
            ("es", "es_MX", Lang::Es),
            ("it", "it_IT", Lang::It),
            ("pt", "pt_PT.UTF-8", Lang::Pt),
            ("pl", "pl_PL", Lang::Pl),
            ("uk", "uk_UA.UTF-8", Lang::Uk),
            ("tr", "tr_TR", Lang::Tr),
            ("zh", "zh_TW.UTF-8", Lang::Zh),
            ("ja", "ja_JP.UTF-8", Lang::Ja),
            ("ko", "ko_KR.UTF-8", Lang::Ko),
        ] {
            assert_eq!(resolve_language(Some(code), None), lang, "{code}");
            assert_eq!(resolve_language(Some("system"), Some(locale)), lang, "{locale}");
            assert_eq!(lang.code(), code);
        }
        assert_eq!(resolve_language(Some("system"), Some("sv_SE.UTF-8")), Lang::En);
    }

    #[test]
    fn every_language_has_its_notifications_and_tray() {
        for lang in Lang::ALL {
            for status in ["completed", "failed"] {
                let (title, body) = run_notification(status, lang).unwrap();
                assert!(!title.is_empty() && !body.is_empty(), "{lang:?} {status}");
            }
            let (title, body) = budget_notification(80, 8.0, 10.0, lang);
            assert!(title.contains("80") && body.contains("$8.00") && body.contains("$10.00"), "{lang:?}: {title} / {body}");
            let (reached, body) = budget_notification(100, 12.0, 10.0, lang);
            assert!(!reached.contains("100") && body.contains("$12.00") && body.len() > budget_notification(80, 12.0, 10.0, lang).1.len(), "{lang:?}");
            let (show, quit) = tray_labels(lang);
            assert!(!show.is_empty() && !quit.is_empty());
            if lang != Lang::En {
                assert_ne!(run_notification("completed", lang), run_notification("completed", Lang::En), "{lang:?}");
                assert_ne!(tray_labels(lang), tray_labels(Lang::En), "{lang:?}");
            }
        }
    }

    #[test]
    fn budget_warnings_speak_the_language() {
        assert_eq!(
            budget_notification(80, 0.85, 1.0, Lang::En),
            ("Monthly budget at 80%".to_string(), "Spent $0.85 of $1.00 this month.".to_string())
        );
        assert_eq!(
            budget_notification(100, 12.346, 10.0, Lang::Ru),
            ("Месячный бюджет исчерпан".to_string(), "Потрачено $12.35 из $10.00 в этом месяце. Запуски не останавливаются.".to_string())
        );
        assert_eq!(budget_notification(80, 8.0, 10.0, Lang::Ru).0, "Месячный бюджет: 80%");
        assert_eq!(budget_notification(100, 1.0, 1.0, Lang::En).0, "Monthly budget reached");
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
