use crate::models::{NormalizationResult, ParseDiagnostic, SourceSpan};

const DESTINATIONS_TO_SKIP: &[&str] = &[
    "fonttbl",
    "colortbl",
    "expandedcolortbl",
    "stylesheet",
    "info",
];

pub fn normalize_text(raw_text: &str) -> NormalizationResult {
    if looks_like_rtf(raw_text) {
        normalize_rtf(raw_text)
    } else {
        NormalizationResult {
            normalized_text: normalize_plain_text(raw_text),
            diagnostics: Vec::new(),
        }
    }
}

fn looks_like_rtf(raw_text: &str) -> bool {
    raw_text.trim_start().starts_with("{\\rtf")
}

fn normalize_rtf(raw_text: &str) -> NormalizationResult {
    let mut diagnostics = Vec::new();
    let mut output = String::new();
    let mut stack = vec![false];
    let mut ignorable = false;
    let mut chars = raw_text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => stack.push(ignorable),
            '}' => {
                ignorable = stack.pop().unwrap_or(false);
            }
            '\\' => handle_control_sequence(&mut chars, &mut output, &mut ignorable),
            '\r' => {}
            '\n' => {
                if !ignorable {
                    output.push('\n');
                }
            }
            _ => {
                if !ignorable {
                    output.push(ch);
                }
            }
        }
    }

    if stack.len() != 1 {
        diagnostics.push(ParseDiagnostic::warning(
            "rtf-unbalanced-groups",
            "RTF group nesting ended in an unexpected state.",
            Some(SourceSpan::new(0, raw_text.len())),
        ));
    }

    NormalizationResult {
        normalized_text: normalize_plain_text(&output),
        diagnostics,
    }
}

fn handle_control_sequence<I>(
    chars: &mut std::iter::Peekable<I>,
    output: &mut String,
    ignorable: &mut bool,
) where
    I: Iterator<Item = char>,
{
    let Some(next) = chars.next() else {
        return;
    };

    match next {
        '\\' | '{' | '}' => {
            if !*ignorable {
                output.push(next);
            }
        }
        '\'' => {
            let hi = chars.next().unwrap_or('0');
            let lo = chars.next().unwrap_or('0');
            if let Ok(byte) = u8::from_str_radix(&format!("{hi}{lo}"), 16) {
                if !*ignorable {
                    output.push(byte as char);
                }
            }
        }
        '*' => *ignorable = true,
        '~' => {
            if !*ignorable {
                output.push('\u{00a0}');
            }
        }
        '-' => {}
        '_' => {
            if !*ignorable {
                output.push('-');
            }
        }
        '\n' => {
            if !*ignorable {
                output.push('\n');
            }
        }
        '\r' => {
            if matches!(chars.peek(), Some('\n')) {
                chars.next();
            }
            if !*ignorable {
                output.push('\n');
            }
        }
        control if control.is_ascii_alphabetic() => {
            let mut word = String::from(control);
            while let Some(peek) = chars.peek() {
                if peek.is_ascii_alphabetic() {
                    word.push(*peek);
                    chars.next();
                } else {
                    break;
                }
            }

            let mut numeric_arg = String::new();
            if matches!(chars.peek(), Some('-')) {
                numeric_arg.push('-');
                chars.next();
            }
            while let Some(peek) = chars.peek() {
                if peek.is_ascii_digit() {
                    numeric_arg.push(*peek);
                    chars.next();
                } else {
                    break;
                }
            }

            if matches!(chars.peek(), Some(' ')) {
                chars.next();
            }

            if DESTINATIONS_TO_SKIP.contains(&word.as_str()) {
                *ignorable = true;
                return;
            }

            if *ignorable {
                return;
            }

            match word.as_str() {
                "par" | "line" => output.push('\n'),
                "tab" => output.push('\t'),
                "u" => {
                    if let Ok(codepoint) = numeric_arg.parse::<i32>() {
                        let normalized = if codepoint < 0 {
                            (codepoint + 65_536) as u32
                        } else {
                            codepoint as u32
                        };
                        if let Some(decoded) = char::from_u32(normalized) {
                            output.push(decoded);
                        }
                    }
                    if matches!(chars.peek(), Some('?')) {
                        chars.next();
                    }
                }
                _ => {}
            }
        }
        other => {
            if !*ignorable {
                output.push(other);
            }
        }
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
