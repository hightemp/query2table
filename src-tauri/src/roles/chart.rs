//! The ```chart block of research answers: a small JSON scheme the frontend draws with ECharts.
//! `check` mirrors `parseChart` in `src/lib/utils/chart.ts` (same order of checks and messages,
//! held together by `tests/fixtures/charts.json`), so the agent can fix what would not draw.

use serde_json::{Map, Value};

pub const TYPES: [&str; 8] = ["bar", "line", "area", "pie", "donut", "scatter", "radar", "heatmap"];
const MAX_SERIES: usize = 12;
const MAX_LABELS: usize = 200;
const MAX_POINTS: usize = 2000;
const MAX_CELLS: usize = 2500;

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

/// Text fields and flags: absent or null is fine, anything else must have its kind.
fn check_fields(chart: &Map<String, Value>) -> Result<(), String> {
    for field in ["title", "x_title", "y_title", "unit", "unit_right"] {
        if chart.get(field).is_some_and(|v| !v.is_null() && !v.is_string()) {
            return Err(format!("\"{field}\" must be text"));
        }
    }
    for field in ["stacked", "horizontal"] {
        if chart.get(field).is_some_and(|v| !v.is_null() && !v.is_boolean()) {
            return Err(format!("\"{field}\" must be true or false"));
        }
    }
    Ok(())
}

/// Number of labels in `field`, which must list text or numbers.
fn labels(chart: &Map<String, Value>, field: &str, missing: &str) -> Result<usize, String> {
    let list = match chart.get(field) {
        Some(Value::Array(list)) if !list.is_empty() => list,
        _ => return Err(missing.to_string()),
    };
    if list.iter().any(|label| !label.is_string() && !label.is_number()) {
        return Err(format!("\"{field}\" must be a list of text labels"));
    }
    if list.len() > MAX_LABELS {
        return Err(format!("at most {MAX_LABELS} labels in \"{field}\""));
    }
    Ok(list.len())
}

/// Checks a chart block; the error says what to fix.
pub fn check(source: &str) -> Result<(), String> {
    let value: Value = serde_json::from_str(source).map_err(|e| format!("not valid JSON: {e}"))?;
    let chart = value.as_object().ok_or("must be a JSON object")?;
    let kind = chart.get("type").and_then(Value::as_str).ok_or("\"type\" is missing")?;
    if !TYPES.contains(&kind) {
        return Err(format!("unknown type \"{kind}\"; use one of {}", TYPES.join(", ")));
    }
    check_fields(chart)?;
    let series = match chart.get("series") {
        Some(Value::Array(series)) if !series.is_empty() => series,
        _ => return Err("\"series\" must list at least one series".into()),
    };
    if series.len() > MAX_SERIES {
        return Err(format!("at most {MAX_SERIES} series"));
    }
    let columns = if kind == "scatter" { 0 } else { labels(chart, "x", "\"x\" must list the labels")? };
    let rows = if kind == "heatmap" { labels(chart, "y", "\"y\" must list the row labels")? } else { 0 };
    match kind {
        "pie" | "donut" if series.len() != 1 => return Err("a pie chart takes one series".into()),
        "heatmap" if series.len() != 1 => return Err("a heatmap takes one series".into()),
        "heatmap" if rows * columns > MAX_CELLS => return Err(format!("at most {MAX_CELLS} cells")),
        "radar" if columns < 3 => return Err("a radar chart needs at least 3 labels in \"x\"".into()),
        _ => {}
    }
    let mixed = matches!(kind, "bar" | "line");
    for (index, item) in series.iter().enumerate() {
        let number = index + 1;
        let item = item.as_object().ok_or_else(|| format!("series {number} must be an object with \"data\""))?;
        let name = match item.get("name") {
            None | Some(Value::Null) => "",
            Some(Value::String(name)) => name.trim(),
            Some(_) => return Err(format!("series {number}: \"name\" must be text")),
        };
        let label = if name.is_empty() { format!("series {number}") } else { format!("series \"{name}\"") };
        let data = item.get("data").and_then(Value::as_array).ok_or_else(|| format!("{label}: \"data\" must be a list"))?;
        if let Some(own) = item.get("type").filter(|v| !v.is_null()) {
            if !mixed {
                return Err("series types are only for bar and line charts".into());
            }
            if !matches!(own.as_str(), Some("bar" | "line")) {
                return Err(format!("{label}: \"type\" must be bar or line"));
            }
        }
        if let Some(axis) = item.get("axis").filter(|v| !v.is_null()) {
            if !mixed {
                return Err("\"axis\" is only for bar and line charts".into());
            }
            if !matches!(axis.as_str(), Some("left" | "right")) {
                return Err(format!("{label}: \"axis\" must be left or right"));
            }
        }
        match kind {
            "scatter" => {
                if data.len() > MAX_POINTS {
                    return Err(format!("{label}: at most {MAX_POINTS} points"));
                }
                for (i, point) in data.iter().enumerate() {
                    if !point.as_array().is_some_and(|p| p.len() == 2 && p.iter().all(Value::is_number)) {
                        return Err(format!("{label}: point {} must be [x, y] numbers", i + 1));
                    }
                }
            }
            "heatmap" => {
                let shaped = data.len() == rows && data.iter().all(|row| row.as_array().is_some_and(|r| r.len() == columns));
                if !shaped {
                    return Err(format!("{label} must have {} of {}", plural(rows, "row"), plural(columns, "value")));
                }
                for (r, row) in data.iter().enumerate() {
                    for (c, cell) in row.as_array().into_iter().flatten().enumerate() {
                        if !cell.is_null() && !cell.is_number() {
                            return Err(format!("{label}: row {}, value {} is not a number", r + 1, c + 1));
                        }
                    }
                }
            }
            _ => {
                if data.len() != columns {
                    return Err(format!("{label} has {} for {}", plural(data.len(), "value"), plural(columns, "label")));
                }
                let pie = matches!(kind, "pie" | "donut");
                for (i, value) in data.iter().enumerate() {
                    if !value.is_number() && !(value.is_null() && !pie) {
                        return Err(format!("{label}: value {} is not a number", i + 1));
                    }
                    if pie && value.as_f64().is_some_and(|v| v < 0.0) {
                        return Err(format!("{label}: value {} is negative", i + 1));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Opening fence of a code block: its character, length and language (first word of the info).
fn fence_open(line: &str) -> Option<(char, usize, String)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let rest = &line[indent..];
    let mark = rest.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let length = rest.len() - rest.trim_start_matches(mark).len();
    let info = rest[length..].trim();
    if indent > 3 || length < 3 || (mark == '`' && info.contains('`')) {
        return None;
    }
    Some((mark, length, info.split_whitespace().next().unwrap_or("").to_ascii_lowercase()))
}

fn fence_closes(line: &str, mark: char, length: usize) -> bool {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let rest = line[indent..].trim_end();
    indent <= 3 && rest.len() >= length && rest.chars().all(|c| c == mark)
}

/// Contents of the top-level fenced code blocks in `language`; blocks inside other code blocks
/// are text of those blocks.
pub fn fenced_blocks(markdown: &str, language: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut open: Option<(char, usize, bool, Vec<&str>)> = None;
    for line in markdown.lines() {
        match &mut open {
            Some((mark, length, wanted, lines)) => {
                if fence_closes(line, *mark, *length) {
                    if *wanted {
                        blocks.push(lines.join("\n"));
                    }
                    open = None;
                } else {
                    lines.push(line);
                }
            }
            None => {
                if let Some((mark, length, info)) = fence_open(line) {
                    open = Some((mark, length, info == language, Vec::new()));
                }
            }
        }
    }
    if let Some((_, _, true, lines)) = open {
        blocks.push(lines.join("\n"));
    }
    blocks
}

/// What is wrong with the answer's charts, as "Chart N: problem" (N counts the chart blocks).
pub fn problems(markdown: &str) -> Vec<String> {
    fenced_blocks(markdown, "chart")
        .iter()
        .enumerate()
        .filter_map(|(i, block)| check(block).err().map(|e| format!("Chart {}: {e}", i + 1)))
        .collect()
}
