//! Splits a file's text into fragments a model can read in one go, keeping each fragment's place.

use super::model::{Fragment, Locator};

/// Target fragment size in characters (roughly 750 tokens).
pub const FRAGMENT_CHARS: usize = 3000;

/// Joins consecutive pieces of text with the same place into fragments of up to `max` characters,
/// and splits longer ones at paragraph, line or word boundaries. Pieces from different places
/// (pages, sections) are never mixed, so every fragment can be cited precisely.
pub fn fragments(blocks: Vec<(Locator, String)>, max: usize) -> Vec<Fragment> {
    let mut out: Vec<Fragment> = Vec::new();
    for (locator, text) in blocks {
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        for piece in split(text, max) {
            match out.last_mut() {
                Some(last) if last.locator == locator && last.text.chars().count() + piece.chars().count() + 2 <= max => {
                    last.text.push_str("\n\n");
                    last.text.push_str(&piece);
                }
                _ => out.push(Fragment { locator: locator.clone(), text: piece }),
            }
        }
    }
    out
}

/// Splits text into pieces of at most `max` characters at the most natural boundary available.
fn split(text: &str, max: usize) -> Vec<String> {
    if text.chars().count() <= max {
        return vec![text.to_string()];
    }
    for separator in ["\n\n", "\n", ". ", " "] {
        let parts: Vec<&str> = text.split(separator).collect();
        if parts.len() < 2 {
            continue;
        }
        let mut pieces = Vec::new();
        let mut current = String::new();
        for part in parts {
            let joined = if current.is_empty() { part.chars().count() } else { current.chars().count() + separator.len() + part.chars().count() };
            if joined > max && !current.is_empty() {
                pieces.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push_str(separator);
            }
            current.push_str(part);
        }
        if !current.is_empty() {
            pieces.push(current);
        }
        return pieces.into_iter().flat_map(|piece| split(piece.trim(), max)).filter(|p| !p.is_empty()).collect();
    }
    // One huge word: cut by characters.
    let chars: Vec<char> = text.chars().collect();
    chars.chunks(max).map(|c| c.iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_short_pieces_of_one_place_and_keeps_places_apart() {
        let intro = Locator { section: Some("Intro".into()), ..Locator::default() };
        let blocks = vec![
            (intro.clone(), "First paragraph.".to_string()),
            (intro.clone(), "Second paragraph.".to_string()),
            (Locator::page(2), "On page two.".to_string()),
            (Locator::page(3), "   ".to_string()),
        ];
        let out = fragments(blocks, 100);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].text, "First paragraph.\n\nSecond paragraph.");
        assert_eq!(out[0].locator, intro);
        assert_eq!(out[1].locator, Locator::page(2));
    }

    #[test]
    fn splits_long_text_at_paragraphs_then_words() {
        let paragraph = "word ".repeat(30);
        let text = format!("{paragraph}\n\n{paragraph}\n\n{paragraph}");
        let out = fragments(vec![(Locator::page(1), text)], 200);
        assert!(out.len() >= 3);
        assert!(out.iter().all(|f| f.text.chars().count() <= 200 && f.locator == Locator::page(1)));
        let long_word = "x".repeat(450);
        let out = fragments(vec![(Locator::page(1), long_word)], 200);
        assert_eq!(out.iter().map(|f| f.text.len()).collect::<Vec<_>>(), vec![200, 200, 50]);
    }
}
