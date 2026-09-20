use crate::providers::accounting::PriceQuote;
use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use scraper::{Html, Selector};
use std::collections::HashMap;

pub(super) async fn limited_text(mut response: reqwest::Response) -> Option<String> {
    const LIMIT: usize = 8 * 1024 * 1024;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > LIMIT as u64)
    {
        return None;
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if body.len() + chunk.len() > LIMIT {
            return None;
        }
        body.extend_from_slice(&chunk);
    }
    String::from_utf8(body).ok()
}
fn number(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.parse().ok())
        .filter(|price| price.is_finite() && *price >= 0.0)
}
pub(super) fn openrouter_catalog(value: &serde_json::Value) -> HashMap<String, PriceQuote> {
    value
        .get("data")
        .and_then(|data| data.as_array())
        .into_iter()
        .flatten()
        .filter_map(|model| {
            let pricing = model.get("pricing")?;
            let quote = PriceQuote {
                input_per_million: number(pricing.get("prompt")?)? * 1_000_000.0,
                output_per_million: number(pricing.get("completion")?)? * 1_000_000.0,
                cached_input_per_million: pricing
                    .get("input_cache_read")
                    .and_then(number)
                    .map(|price| price * 1_000_000.0),
                cache_write_per_million: pricing
                    .get("input_cache_write")
                    .and_then(number)
                    .map(|price| price * 1_000_000.0),
                per_request: match pricing.get("request") {
                    Some(value) => number(value)?,
                    None => 0.0,
                },
                source: "OpenRouter model catalog".into(),
                credit_based: false,
            };
            let id = model.get("id")?.as_str()?;
            quote.valid().then(|| (id.to_string(), quote))
        })
        .collect()
}

pub(super) struct OllamaPrices {
    base: HashMap<String, PriceQuote>,
    peak: HashMap<String, PriceQuote>,
}
impl OllamaPrices {
    pub fn parse(html: &str) -> Option<Self> {
        let document = Html::parse_document(html);
        let selector = |text| Selector::parse(text).ok();
        let section = document
            .select(&selector("section#model-pricing")?)
            .next()?;
        let text = section
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !text.contains("Prices are per million tokens.")
            || !text.contains("Peak pricing applies between 12:00 and 18:00 UTC, Monday to Friday.")
        {
            return None;
        }
        let tables: Vec<_> = section.select(&selector("table")?).collect();
        if tables.len() != 2 {
            return None;
        }
        let mut catalogs = Vec::new();
        for (index, table) in tables.iter().enumerate() {
            let headers: Vec<_> = table
                .select(&selector("thead th")?)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();
            if headers != ["Model", "Input", "Cached input", "Output"] {
                return None;
            }
            let mut catalog = HashMap::new();
            for row in table.select(&selector("tbody tr")?) {
                let cells: Vec<_> = row
                    .select(&selector("td")?)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .collect();
                if cells.len() != 4 || cells[0].is_empty() {
                    return None;
                }
                let usd = |text: &str| -> Option<f64> {
                    text.strip_prefix('$')?
                        .trim()
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite() && *value >= 0.0)
                };
                let quote = PriceQuote {
                    input_per_million: usd(&cells[1])?,
                    output_per_million: usd(&cells[3])?,
                    cached_input_per_million: if cells[2] == "-" {
                        None
                    } else {
                        Some(usd(&cells[2])?)
                    },
                    cache_write_per_million: None,
                    per_request: 0.0,
                    source: if index == 0 {
                        "Ollama published rates".into()
                    } else {
                        "Ollama published peak rates".into()
                    },
                    credit_based: true,
                };
                if catalog.insert(cells[0].clone(), quote).is_some() {
                    return None;
                }
            }
            if catalog.is_empty() {
                return None;
            }
            catalogs.push(catalog);
        }
        let peak = catalogs.pop()?;
        let base = catalogs.pop()?;
        if peak.keys().any(|model| !base.contains_key(model)) {
            return None;
        }
        Some(Self { base, peak })
    }
    pub fn quote(&self, model: &str, at: DateTime<Utc>) -> Option<PriceQuote> {
        let model = model
            .strip_suffix(":cloud")
            .or_else(|| model.strip_suffix("-cloud"))
            .or_else(|| model.strip_suffix(":latest"))
            .unwrap_or(model);
        let peak =
            !matches!(at.weekday(), Weekday::Sat | Weekday::Sun) && (12..18).contains(&at.hour());
        if peak {
            self.peak
                .get(model)
                .or_else(|| self.base.get(model))
                .cloned()
        } else {
            self.base.get(model).cloned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    fn page() -> String {
        let table = |price: &str| {
            format!("<table><thead><tr><th>Model</th><th>Input</th><th>Cached input</th><th>Output</th></tr></thead><tbody><tr><td>example:120b</td><td>${price}</td><td>$0.05</td><td>$3</td></tr></tbody></table>")
        };
        format!("<section id='model-pricing'><p>Prices are per million tokens.</p>{}<h3>Peak pricing</h3><p>Peak pricing applies between 12:00 and 18:00 UTC, Monday to Friday.</p>{}</section>",table("1"),table("2"))
    }
    #[test]
    fn cloud_rates_respect_peak_boundaries_weekends_and_cloud_aliases() {
        let prices = OllamaPrices::parse(&page()).unwrap();
        for (day, hour, expected) in [
            (18, 11, 1.0),
            (18, 12, 2.0),
            (18, 17, 2.0),
            (18, 18, 1.0),
            (19, 13, 1.0),
            (20, 13, 1.0),
        ] {
            let at = Utc.with_ymd_and_hms(2026, 9, day, hour, 0, 0).unwrap();
            assert_eq!(
                prices
                    .quote("example:120b-cloud", at)
                    .unwrap()
                    .input_per_million,
                expected
            );
        }
        assert!(prices.quote("unknown", Utc::now()).is_none());
    }
    #[test]
    fn changed_html_or_unknown_peak_schedule_fails_closed_to_unknown_prices() {
        assert!(OllamaPrices::parse(&page().replace("12:00", "11:00")).is_none());
        assert!(OllamaPrices::parse(&page().replace("Cached input", "Other unit")).is_none());
        assert!(OllamaPrices::parse(&page().replace("$3", "$NaN")).is_none());
    }
    #[test]
    fn router_prices_use_per_token_units_and_keep_free_models() {
        let prices = openrouter_catalog(
            &serde_json::json!({"data":[{"id":"free","pricing":{"prompt":"0","completion":"0"}},{"id":"paid","pricing":{"prompt":"0.000002","completion":"0.000008","input_cache_read":"0.0000005"}},{"id":"invalid","pricing":{"prompt":"-1","completion":"1"}}]}),
        );
        assert_eq!(prices["free"].input_per_million, 0.0);
        assert_eq!(prices["paid"].input_per_million, 2.0);
        assert_eq!(prices["paid"].cached_input_per_million, Some(0.5));
        assert!(!prices.contains_key("invalid"));
        let invalid_fee = serde_json::json!({"data": [{"id": "invalid-fee", "pricing": {
            "prompt": "0", "completion": "0", "request": "unknown"
        }}]});
        assert!(openrouter_catalog(&invalid_fee).is_empty());
    }
}
