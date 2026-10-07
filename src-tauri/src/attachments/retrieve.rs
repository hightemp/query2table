//! Finds the fragments of attached files that matter for a task (SQLite FTS5, BM25 ranking).

use sqlx::{Row, SqlitePool};

use super::model::Locator;

/// A fragment found for a task, ready to cite by `url`.
#[derive(Debug, Clone, PartialEq)]
pub struct FragmentHit {
    pub attachment_id: String,
    pub file_name: String,
    pub locator: Locator,
    pub url: String,
    pub text: String,
}

/// Words too common to help ranking.
const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "from", "that", "this", "what", "which", "are", "was", "how", "about", "into", "find",
    "list", "all", "any", "его", "она", "они", "что", "как", "для", "это", "или", "все", "при", "так", "найди", "найти",
    "список", "мне", "над", "под", "без", "про",
];

/// An FTS5 query matching any meaningful word of `text`; long words also match by their stem,
/// so "компаний" finds "компании". None when nothing is left to search for.
pub fn fts_query(text: &str) -> Option<String> {
    let mut terms: Vec<String> = Vec::new();
    for word in text.split(|c: char| !c.is_alphanumeric()).map(str::to_lowercase) {
        let chars: Vec<char> = word.chars().collect();
        let numeric = chars.iter().all(char::is_ascii_digit);
        if (chars.len() < 3 && !(numeric && chars.len() >= 2)) || STOP_WORDS.contains(&word.as_str()) {
            continue;
        }
        let term = if chars.len() >= 6 && !numeric {
            format!("\"{}\"*", chars[..chars.len() - 2].iter().collect::<String>())
        } else {
            format!("\"{word}\"")
        };
        if !terms.contains(&term) {
            terms.push(term);
        }
        if terms.len() == 32 {
            break;
        }
    }
    (!terms.is_empty()).then(|| terms.join(" OR "))
}

/// The best fragments of the given files for `query`, best first.
pub async fn search(pool: &SqlitePool, attachment_ids: &[String], query: &str, limit: usize) -> Result<Vec<FragmentHit>, sqlx::Error> {
    let Some(matcher) = fts_query(query) else { return Ok(Vec::new()) };
    if attachment_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; attachment_ids.len()].join(", ");
    let sql = format!(
        "SELECT c.attachment_id, a.file_name, c.locator, c.text
         FROM attachment_chunks_fts f
         JOIN attachment_chunks c ON c.id = f.rowid
         JOIN attachments a ON a.id = c.attachment_id
         WHERE attachment_chunks_fts MATCH ? AND c.attachment_id IN ({placeholders})
         ORDER BY bm25(attachment_chunks_fts)
         LIMIT ?"
    );
    let mut q = sqlx::query(&sql).bind(matcher);
    for id in attachment_ids {
        q = q.bind(id);
    }
    let rows = q.bind(limit as i64).fetch_all(pool).await?;
    Ok(rows.into_iter().map(hit).collect())
}

/// Fragments at a place in a file (a page, rows of a sheet, a section), in file order.
/// An empty place gives the beginning of the file.
pub async fn at(pool: &SqlitePool, attachment_id: &str, place: &Locator, limit: usize) -> Result<Vec<FragmentHit>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT c.attachment_id, a.file_name, c.locator, c.text
         FROM attachment_chunks c JOIN attachments a ON a.id = c.attachment_id
         WHERE c.attachment_id = ? ORDER BY c.ord",
    )
    .bind(attachment_id)
    .fetch_all(pool)
    .await?;
    let all: Vec<FragmentHit> = rows.into_iter().map(hit).collect();
    let matching: Vec<FragmentHit> = all.iter().filter(|h| covers(&h.locator, place)).take(limit).cloned().collect();
    if matching.is_empty() && *place == Locator::default() {
        return Ok(all.into_iter().take(limit).collect());
    }
    Ok(matching)
}

fn covers(fragment: &Locator, place: &Locator) -> bool {
    if let Some(page) = place.page {
        return fragment.page == Some(page);
    }
    if let Some(sheet) = &place.sheet {
        if fragment.sheet.as_ref() != Some(sheet) {
            return false;
        }
    }
    if let Some((from, to)) = place.rows {
        return fragment.rows.is_some_and(|(a, b)| a <= to && from <= b);
    }
    if let Some(section) = &place.section {
        return fragment.section.as_ref() == Some(section);
    }
    place.sheet.is_some()
}

fn hit(row: sqlx::sqlite::SqliteRow) -> FragmentHit {
    let attachment_id: String = row.get("attachment_id");
    let locator: Locator = serde_json::from_str(&row.get::<String, _>("locator")).unwrap_or_default();
    FragmentHit {
        url: locator.to_url(&attachment_id),
        attachment_id,
        file_name: row.get("file_name"),
        locator,
        text: row.get("text"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::store::AttachmentStore;
    use crate::attachments::test_files;
    use crate::storage::db::Database;

    #[test]
    fn queries_keep_meaningful_words_and_stem_long_ones() {
        assert_eq!(fts_query("Find the competitors of Acme in 2025").as_deref(), Some("\"competito\"* OR \"acme\" OR \"2025\""));
        assert_eq!(fts_query("Найди цены компаний").as_deref(), Some("\"цены\" OR \"компан\"*"));
        assert_eq!(fts_query("a, of — ?"), None);
        // Quotes in the text cannot break the query.
        assert_eq!(fts_query("say \"hello\" OR").as_deref(), Some("\"say\" OR \"hello\""));
    }

    #[tokio::test]
    async fn finds_matching_fragments_and_places() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        Database::with_pool(pool.clone()).await.migrate().await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = AttachmentStore::new(pool.clone(), dir.path().to_path_buf());
        let report = store
            .add_bytes("report.pdf", test_files::pdf(&["Company history and founders", "Revenue of the company grew strongly", "Office locations"]))
            .await
            .unwrap();
        let mut rows: Vec<Vec<String>> = vec![vec!["Город".into(), "Компания".into()]];
        rows.extend((1..=200).map(|i| vec![format!("Город {i}"), format!("Компания номер {i} с длинным названием")]));
        rows.push(vec!["Казань".into(), "Компании Татарстана".into()]);
        let sheet = store.add_bytes("cities.xlsx", test_files::xlsx(&[("Список", rows)])).await.unwrap();
        let other = store.add_bytes("other.txt", b"Revenue numbers elsewhere".to_vec()).await.unwrap();

        let hits = search(&pool, &[report.id.clone()], "How did revenue grow?", 5).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].locator, Locator::page(2));
        assert_eq!(hits[0].url, format!("attachment://{}?page=2", report.id));
        assert_eq!(hits[0].file_name, "report.pdf");

        let hits = search(&pool, &[sheet.id.clone(), report.id.clone()], "Казань", 5).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].locator.sheet.as_deref(), Some("Список"));
        assert!(hits[0].text.contains("Row 202: Казань"));
        assert!(search(&pool, &[other.id.clone()], "Казань", 5).await.unwrap().is_empty());

        let page = at(&pool, &report.id, &Locator::page(3), 5).await.unwrap();
        assert_eq!(page.len(), 1);
        assert!(page[0].text.contains("Office locations"));
        let rows = at(&pool, &sheet.id, &Locator { sheet: Some("Список".into()), rows: Some((202, 202)), ..Locator::default() }, 5).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].text.contains("Казань"));
        let start = at(&pool, &report.id, &Locator::default(), 1).await.unwrap();
        assert!(start[0].text.contains("Company history"));
    }
}
