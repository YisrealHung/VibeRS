use crate::theme::Palette;
use egui::{
    text::LayoutJob,
    Color32, FontId, TextFormat,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    Rust,
    Cpp,
    Python,
    JavaScript,
    Json,
    Toml,
    PlainText,
}

impl Language {
    pub fn from_file_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.ends_with(".rs") {
            Language::Rust
        } else if lower.ends_with(".cpp")
            || lower.ends_with(".cc")
            || lower.ends_with(".cxx")
            || lower.ends_with(".c")
            || lower.ends_with(".h")
            || lower.ends_with(".hpp")
        {
            Language::Cpp
        } else if lower.ends_with(".py") {
            Language::Python
        } else if lower.ends_with(".js") || lower.ends_with(".ts") || lower.ends_with(".jsx") || lower.ends_with(".tsx") {
            Language::JavaScript
        } else if lower.ends_with(".json") {
            Language::Json
        } else if lower.ends_with(".toml") {
            Language::Toml
        } else {
            Language::PlainText
        }
    }

    pub fn keywords(self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "fn", "pub", "mod", "use", "let", "mut", "struct", "enum", "impl", "for", "in",
                "if", "else", "match", "return", "self", "Self", "super", "crate", "const",
                "static", "where", "trait", "type", "async", "await", "move", "ref", "true",
                "false", "unsafe", "extern", "continue", "break", "loop", "as",
            ],
            Language::Cpp => &[
                "auto", "bool", "break", "case", "catch", "char", "class", "const", "continue",
                "default", "delete", "do", "double", "else", "enum", "explicit", "extern", "false",
                "float", "for", "friend", "goto", "if", "inline", "int", "long", "mutable",
                "namespace", "new", "noexcept", "nullptr", "operator", "private", "protected",
                "public", "register", "reinterpret_cast", "return", "short", "signed", "sizeof",
                "static", "static_cast", "struct", "switch", "template", "this", "throw", "true",
                "try", "typedef", "typeid", "typename", "union", "unsigned", "using", "virtual",
                "void", "volatile", "wchar_t", "while", "override", "final", "include",
            ],
            Language::Python => &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                "del", "elif", "else", "except", "False", "finally", "for", "from", "global",
                "if", "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass",
                "raise", "return", "True", "try", "while", "with", "yield", "self",
            ],
            Language::JavaScript => &[
                "break", "case", "catch", "class", "const", "continue", "debugger", "default",
                "delete", "do", "else", "export", "extends", "finally", "for", "function", "if",
                "import", "in", "instanceof", "new", "return", "super", "switch", "this", "throw",
                "try", "typeof", "var", "void", "while", "with", "yield", "let", "static", "enum",
                "await", "async", "from", "as", "true", "false", "null", "undefined",
            ],
            Language::Json | Language::Toml | Language::PlainText => &[],
        }
    }
}

pub fn highlight_code(text: &str, lang: Language) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font_id = FontId::monospace(13.0);

    if lang == Language::PlainText {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id,
                color: Palette::TEXT_PRIMARY,
                ..Default::default()
            },
        );
        return job;
    }

    let keywords = lang.keywords();

    for (line_idx, line) in text.split('\n').enumerate() {
        if line_idx > 0 {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::TEXT_PRIMARY,
                    ..Default::default()
                },
            );
        }

        let trimmed = line.trim_start();
        // Line comment check
        if trimmed.starts_with("//") || (lang == Language::Python && trimmed.starts_with('#')) || (lang == Language::Toml && trimmed.starts_with('#')) {
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::SYN_COMMENT,
                    ..Default::default()
                },
            );
            continue;
        }

        let mut chars = line.char_indices().peekable();
        while let Some(&(start, ch)) = chars.peek() {
            // Strings: "..." or '...'
            if ch == '"' || ch == '\'' {
                let quote = ch;
                chars.next();
                let mut end = line.len();
                let mut escaped = false;
                for (i, c) in chars.by_ref() {
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == quote {
                        end = i + 1;
                        break;
                    }
                }
                job.append(
                    &line[start..end],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::SYN_STRING,
                        ..Default::default()
                    },
                );
            } else if ch.is_alphabetic() || ch == '_' {
                let mut end = line.len();
                while let Some(&(i, c)) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        chars.next();
                    } else {
                        end = i;
                        break;
                    }
                }
                let word = &line[start..end];
                let color = if keywords.contains(&word) {
                    Palette::SYN_KEYWORD
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    Palette::SYN_TYPE
                } else {
                    Palette::TEXT_PRIMARY
                };
                job.append(
                    word,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
            } else if ch.is_ascii_digit() {
                let mut end = line.len();
                while let Some(&(i, c)) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' || c == 'x' || c == 'f' || c == 'u' || c == 'i' {
                        chars.next();
                    } else {
                        end = i;
                        break;
                    }
                }
                job.append(
                    &line[start..end],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Color32::from_rgb(220, 160, 100),
                        ..Default::default()
                    },
                );
            } else {
                chars.next();
                let next_pos = chars.peek().map(|&(i, _)| i).unwrap_or(line.len());
                job.append(
                    &line[start..next_pos],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::TEXT_PRIMARY,
                        ..Default::default()
                    },
                );
            }
        }
    }

    job
}

