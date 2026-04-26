use crate::models::NormalizationResult;

pub fn normalize_text(raw_text: &str) -> NormalizationResult {
    NormalizationResult {
        normalized_text: normalize_plain_text(raw_text),
        diagnostics: Vec::new(),
    }
}

fn normalize_plain_text(text: &str) -> String {
    let text = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\u{00a0}', " ");
    let mut normalized_lines = Vec::new();
    let mut previous_blank = false;

    for raw_line in text.lines() {
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            if !previous_blank {
                normalized_lines.push(String::new());
            }
            previous_blank = true;
        } else {
            normalized_lines.push(line.to_string());
            previous_blank = false;
        }
    }

    normalized_lines.join("\n").trim().to_string()
}
