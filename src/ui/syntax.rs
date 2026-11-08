use crate::theme::Palette;
use egui::{
    text::LayoutJob,
    FontId, TextFormat,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    Rust,
    Cpp,
    Python,
    JavaScript,
    Java,
    CSharp,
    Go,
    Html,
    Sql,
    Markdown,
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
        } else if lower.ends_with(".js")
            || lower.ends_with(".ts")
            || lower.ends_with(".jsx")
            || lower.ends_with(".tsx")
        {
            Language::JavaScript
        } else if lower.ends_with(".java") {
            Language::Java
        } else if lower.ends_with(".cs") {
            Language::CSharp
        } else if lower.ends_with(".go") {
            Language::Go
        } else if lower.ends_with(".html") || lower.ends_with(".htm") {
            Language::Html
        } else if lower.ends_with(".sql") {
            Language::Sql
        } else if lower.ends_with(".md") || lower.ends_with(".markdown") {
            Language::Markdown
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
            Language::Java => &[
                "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
                "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
                "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
                "interface", "long", "native", "new", "package", "private", "protected", "public",
                "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
                "throw", "throws", "transient", "try", "void", "volatile", "while", "true", "false",
                "null", "record", "var", "yield", "sealed", "permits",
            ],
            Language::CSharp => &[
                "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char",
                "checked", "class", "const", "continue", "decimal", "default", "delegate", "do",
                "double", "else", "enum", "event", "explicit", "extern", "false", "finally",
                "fixed", "float", "for", "foreach", "goto", "if", "implicit", "in", "int",
                "interface", "internal", "is", "lock", "long", "namespace", "new", "null",
                "object", "operator", "out", "override", "params", "private", "protected", "public",
                "readonly", "ref", "return", "sbyte", "sealed", "short", "sizeof", "stackalloc",
                "static", "string", "struct", "switch", "this", "throw", "true", "try", "typeof",
                "uint", "ulong", "unchecked", "unsafe", "ushort", "using", "virtual", "void",
                "volatile", "while", "var", "async", "await", "record", "init", "get", "set",
            ],
            Language::Go => &[
                "break", "default", "func", "interface", "select", "case", "defer", "go", "map",
                "struct", "chan", "else", "goto", "package", "switch", "const", "fallthrough", "if",
                "range", "type", "continue", "for", "import", "return", "var", "true", "false",
                "nil", "iota", "make", "new", "len", "cap", "append", "copy", "close", "delete",
                "panic", "recover", "string", "int", "int64", "int32", "uint", "uint64", "uint32",
                "byte", "rune", "float32", "float64", "bool", "error", "any",
            ],
            Language::Html => &[
                "html", "head", "body", "title", "meta", "link", "style", "script", "div", "span",
                "p", "a", "h1", "h2", "h3", "h4", "h5", "h6", "ul", "ol", "li", "table", "tr", "td",
                "th", "thead", "tbody", "form", "input", "button", "select", "option", "textarea",
                "label", "img", "iframe", "canvas", "svg", "header", "footer", "nav", "section",
                "article", "aside", "main", "code", "pre", "strong", "em", "b", "i", "br", "hr",
                "class", "id", "href", "src", "type", "rel", "name", "value", "placeholder",
                "doctype", "DOCTYPE",
            ],
            Language::Sql => &[
                "select", "from", "where", "insert", "into", "update", "delete", "join", "left",
                "right", "inner", "outer", "full", "cross", "on", "group", "by", "order", "having",
                "limit", "offset", "as", "and", "or", "not", "in", "is", "null", "like", "between",
                "exists", "union", "all", "create", "table", "drop", "alter", "index", "view",
                "database", "schema", "primary", "key", "foreign", "references", "constraint",
                "default", "check", "unique", "auto_increment", "varchar", "int", "bigint", "text",
                "boolean", "date", "datetime", "timestamp", "decimal", "float", "blob", "set",
                "values", "distinct", "count", "sum", "avg", "min", "max", "case", "when", "then",
                "else", "end", "cast", "coalesce", "truncate", "begin", "commit", "rollback",
                "transaction", "grant", "revoke",
            ],
            Language::Json | Language::Toml | Language::Markdown | Language::PlainText => &[],
        }
    }
}

pub fn highlight_code(text: &str, lang: Language) -> LayoutJob {
    if lang == Language::Markdown {
        return highlight_markdown(text);
    }

    let mut job = LayoutJob::default();
    let font_id = FontId::monospace(13.0);

    if lang == Language::PlainText {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id,
                color: Palette::TEXT_PRIMARY(),
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
                    color: Palette::TEXT_PRIMARY(),
                    ..Default::default()
                },
            );
        }

        let trimmed = line.trim_start();
        // Line comment check
        if trimmed.starts_with("//")
            || (lang == Language::Python && trimmed.starts_with('#'))
            || (lang == Language::Toml && trimmed.starts_with('#'))
            || (lang == Language::Sql && trimmed.starts_with("--"))
            || (lang == Language::Html && trimmed.starts_with("<!--"))
        {
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::SYN_COMMENT(),
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
                        color: Palette::SYN_STRING(),
                        ..Default::default()
                    },
                );
            } else if lang == Language::Html && (ch == '<' || ch == '>') {
                chars.next();
                let next_pos = chars.peek().map(|&(i, _)| i).unwrap_or(line.len());
                job.append(
                    &line[start..next_pos],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::ACCENT(),
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
                let is_kw = if lang == Language::Sql || lang == Language::Html {
                    keywords.iter().any(|k| k.eq_ignore_ascii_case(word))
                } else {
                    keywords.contains(&word)
                };
                let color = if is_kw {
                    Palette::SYN_KEYWORD()
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    Palette::SYN_TYPE()
                } else {
                    Palette::TEXT_PRIMARY()
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
                        color: Palette::AMBER(),
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
                        color: Palette::TEXT_PRIMARY(),
                        ..Default::default()
                    },
                );
            }
        }
    }

    job
}

pub fn highlight_markdown(text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mono_font = FontId::monospace(13.0);
    let mut in_code_block = false;

    for (line_idx, line) in text.split('\n').enumerate() {
        if line_idx > 0 {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: mono_font.clone(),
                    color: Palette::TEXT_PRIMARY(),
                    ..Default::default()
                },
            );
        }

        let trimmed = line.trim_start();

        // 1. Code fence ```
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: mono_font.clone(),
                    color: Palette::AMBER(),
                    ..Default::default()
                },
            );
            continue;
        }

        // 2. Inside code block
        if in_code_block {
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: mono_font.clone(),
                    color: Palette::SYN_STRING(),
                    ..Default::default()
                },
            );
            continue;
        }

        // 3. Horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: mono_font.clone(),
                    color: Palette::SYN_COMMENT(),
                    ..Default::default()
                },
            );
            continue;
        }

        // 4. HTML comments
        if trimmed.starts_with("<!--") {
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: mono_font.clone(),
                    color: Palette::SYN_COMMENT(),
                    ..Default::default()
                },
            );
            continue;
        }

        // 5. Headings (# H1, ## H2, ### H3...)
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if hash_count <= 6 && (trimmed.len() == hash_count || trimmed[hash_count..].starts_with(' ')) {
                let leading_ws = &line[..line.len() - trimmed.len()];
                if !leading_ws.is_empty() {
                    job.append(
                        leading_ws,
                        0.0,
                        TextFormat {
                            font_id: mono_font.clone(),
                            color: Palette::TEXT_PRIMARY(),
                            ..Default::default()
                        },
                    );
                }
                let hashes = &trimmed[..hash_count];
                job.append(
                    hashes,
                    0.0,
                    TextFormat {
                        font_id: mono_font.clone(),
                        color: Palette::ACCENT(),
                        ..Default::default()
                    },
                );
                let heading_content = &trimmed[hash_count..];
                job.append(
                    heading_content,
                    0.0,
                    TextFormat {
                        font_id: mono_font.clone(),
                        color: if hash_count <= 2 {
                            Palette::highlight_text()
                        } else {
                            Palette::ACCENT()
                        },
                        ..Default::default()
                    },
                );
                continue;
            }
        }

        // 6. Blockquote (> ...)
        if trimmed.starts_with('>') {
            let leading_ws = &line[..line.len() - trimmed.len()];
            if !leading_ws.is_empty() {
                job.append(
                    leading_ws,
                    0.0,
                    TextFormat {
                        font_id: mono_font.clone(),
                        color: Palette::TEXT_PRIMARY(),
                        ..Default::default()
                    },
                );
            }
            job.append(
                ">",
                0.0,
                TextFormat {
                    font_id: mono_font.clone(),
                    color: Palette::ACCENT(),
                    ..Default::default()
                },
            );
            let rest = &trimmed[1..];
            highlight_markdown_inline(rest, &mut job, &mono_font);
            continue;
        }

        // 7. Checkboxes and list items
        let (is_list, bullet_len, bullet_col) = if trimmed.starts_with("- [x] ")
            || trimmed.starts_with("- [X] ")
            || trimmed.starts_with("* [x] ")
            || trimmed.starts_with("* [X] ")
        {
            (true, 6, Palette::GREEN())
        } else if trimmed.starts_with("- [ ] ") || trimmed.starts_with("* [ ] ") {
            (true, 6, Palette::AMBER())
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            (true, 2, Palette::ACCENT())
        } else if trimmed.len() > 3
            && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
            && trimmed.contains(". ")
        {
            let dot_pos = trimmed.find(". ").unwrap();
            (true, dot_pos + 2, Palette::AMBER())
        } else {
            (false, 0, Palette::TEXT_PRIMARY())
        };

        if is_list {
            let leading_ws = &line[..line.len() - trimmed.len()];
            if !leading_ws.is_empty() {
                job.append(
                    leading_ws,
                    0.0,
                    TextFormat {
                        font_id: mono_font.clone(),
                        color: Palette::TEXT_PRIMARY(),
                        ..Default::default()
                    },
                );
            }
            let bullet = &trimmed[..bullet_len];
            job.append(
                bullet,
                0.0,
                TextFormat {
                    font_id: mono_font.clone(),
                    color: bullet_col,
                    ..Default::default()
                },
            );
            let rest = &trimmed[bullet_len..];
            highlight_markdown_inline(rest, &mut job, &mono_font);
            continue;
        }

        // 8. Normal line with inline markdown elements
        highlight_markdown_inline(line, &mut job, &mono_font);
    }

    job
}

fn highlight_markdown_inline(text: &str, job: &mut LayoutJob, font_id: &FontId) {
    let mut chars = text.char_indices().peekable();
    let mut last_idx = 0;

    while let Some(&(i, ch)) = chars.peek() {
        // 1. Inline code: `...`
        if ch == '`' {
            if i > last_idx {
                job.append(
                    &text[last_idx..i],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::TEXT_PRIMARY(),
                        ..Default::default()
                    },
                );
            }
            chars.next();
            let code_start = i;
            let mut code_end = None;
            for (idx, c) in chars.by_ref() {
                if c == '`' {
                    code_end = Some(idx + 1);
                    break;
                }
            }
            let end = code_end.unwrap_or(text.len());
            job.append(
                &text[code_start..end],
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::inline_code_text(),
                    background: Palette::inline_code_bg(),
                    ..Default::default()
                },
            );
            last_idx = end;
        }
        // 2. Bold: **...** or __...__
        else if (ch == '*' && text[i..].starts_with("**")) || (ch == '_' && text[i..].starts_with("__")) {
            let delim = &text[i..i + 2];
            if i > last_idx {
                job.append(
                    &text[last_idx..i],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::TEXT_PRIMARY(),
                        ..Default::default()
                    },
                );
            }
            chars.next();
            chars.next();
            let mut end_pos = None;
            let remaining = &text[i + 2..];
            if let Some(pos) = remaining.find(delim) {
                let target = i + 2 + pos + 2;
                end_pos = Some(target);
                while let Some(&(next_i, _)) = chars.peek() {
                    if next_i < target {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            let end = end_pos.unwrap_or(text.len());
            job.append(
                &text[i..end],
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::highlight_text(),
                    ..Default::default()
                },
            );
            last_idx = end;
        }
        // 3. Image: ![alt](url)
        else if ch == '!' && text[i..].starts_with("![") {
            if i > last_idx {
                job.append(
                    &text[last_idx..i],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::TEXT_PRIMARY(),
                        ..Default::default()
                    },
                );
            }
            if let Some(close_bracket) = text[i..].find("](") {
                if let Some(close_paren) = text[i + close_bracket..].find(')') {
                    let end = i + close_bracket + close_paren + 1;
                    while let Some(&(next_i, _)) = chars.peek() {
                        if next_i < end {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    job.append(
                        "!",
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Palette::AMBER(),
                            ..Default::default()
                        },
                    );
                    job.append(
                        &text[i + 1..end],
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Palette::ACCENT(),
                            ..Default::default()
                        },
                    );
                    last_idx = end;
                    continue;
                }
            }
            chars.next();
        }
        // 4. Link: [text](url)
        else if ch == '[' {
            if let Some(close_bracket) = text[i..].find("](") {
                if let Some(close_paren) = text[i + close_bracket..].find(')') {
                    let end = i + close_bracket + close_paren + 1;
                    if i > last_idx {
                        job.append(
                            &text[last_idx..i],
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color: Palette::TEXT_PRIMARY(),
                                ..Default::default()
                            },
                        );
                    }
                    while let Some(&(next_i, _)) = chars.peek() {
                        if next_i < end {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let bracket_end = i + close_bracket + 1;
                    job.append(
                        &text[i..bracket_end],
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Palette::ACCENT(),
                            ..Default::default()
                        },
                    );
                    job.append(
                        &text[bracket_end..end],
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: Palette::TEXT_MUTED(),
                            ..Default::default()
                        },
                    );
                    last_idx = end;
                    continue;
                }
            }
            chars.next();
        }
        // 5. Table bar: |
        else if ch == '|' {
            if i > last_idx {
                job.append(
                    &text[last_idx..i],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::TEXT_PRIMARY(),
                        ..Default::default()
                    },
                );
            }
            chars.next();
            job.append(
                "|",
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::ACCENT(),
                    ..Default::default()
                },
            );
            last_idx = i + 1;
        } else {
            chars.next();
        }
    }

    if last_idx < text.len() {
        job.append(
            &text[last_idx..],
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: Palette::TEXT_PRIMARY(),
                ..Default::default()
            },
        );
    }
}

