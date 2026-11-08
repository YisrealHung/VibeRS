use crate::theme::Palette;
use egui::{text::LayoutJob, CornerRadius, FontId, Frame, Margin, RichText, Stroke, TextFormat, Ui};

/// Render beautiful Markdown blocks like Antigravity IDE
pub fn render_markdown_message(ui: &mut Ui, markdown: &str) {
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();

    let mut lines = markdown.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // 1. Code blocks (```lang ... ```)
        if trimmed.starts_with("```") {
            if in_code_block {
                // End code block
                render_code_card(ui, &code_block_lang, &code_block_content);
                code_block_content.clear();
                code_block_lang.clear();
                in_code_block = false;
            } else {
                // Start code block
                in_code_block = true;
                code_block_lang = trimmed.trim_start_matches('`').trim().to_string();
            }
            continue;
        }

        if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
            continue;
        }

        // 2. Horizontal divider (--- or ***)
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            continue;
        }

        // 3. Headings (# H1, ## H2, ### H3, #### H4)
        if let Some(rest) = line.strip_prefix("###### ") {
            ui.add_space(6.0);
            ui.label(RichText::new(clean_markdown_inline(rest)).size(12.0).strong().color(Palette::TEXT_MUTED()));
            ui.add_space(3.0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("##### ") {
            ui.add_space(6.0);
            ui.label(RichText::new(clean_markdown_inline(rest)).size(12.5).strong().color(Palette::TEXT_PRIMARY()));
            ui.add_space(3.0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("#### ") {
            ui.add_space(8.0);
            ui.label(RichText::new(clean_markdown_inline(rest)).size(13.0).strong().color(Palette::TEXT_PRIMARY()));
            ui.add_space(4.0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            ui.add_space(10.0);
            ui.label(RichText::new(clean_markdown_inline(rest)).size(14.0).strong().color(Palette::ACCENT()));
            ui.add_space(4.0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            ui.add_space(12.0);
            ui.label(RichText::new(clean_markdown_inline(rest)).size(15.5).strong().color(Palette::highlight_text()));
            ui.add_space(5.0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            ui.add_space(14.0);
            ui.label(RichText::new(clean_markdown_inline(rest)).size(18.0).strong().color(Palette::ACCENT()));
            ui.add_space(6.0);
            continue;
        }

        // 4. Tables: | Header | Header | ...
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            if let Some(&next_line) = lines.peek() {
                let next_trim = next_line.trim();
                if next_trim.starts_with('|') && next_trim.contains('-') && next_trim.ends_with('|') {
                    // This is a Markdown table!
                    let header_line = trimmed;
                    let _divider = lines.next(); // consume divider

                    let headers: Vec<&str> = header_line
                        .trim_matches('|')
                        .split('|')
                        .map(|s| s.trim())
                        .collect();

                    let mut data_rows: Vec<Vec<String>> = Vec::new();
                    while let Some(&row_line) = lines.peek() {
                        let row_trim = row_line.trim();
                        if row_trim.starts_with('|') && row_trim.ends_with('|') {
                            let cells: Vec<String> = row_trim
                                .trim_matches('|')
                                .split('|')
                                .map(|s| s.trim().to_string())
                                .collect();
                            data_rows.push(cells);
                            lines.next();
                        } else {
                            break;
                        }
                    }

                    render_markdown_table(ui, &headers, &data_rows);
                    continue;
                }
            }
        }

        // 5. Blockquotes (> ...)
        if let Some(rest) = line.strip_prefix("> ") {
            ui.add_space(4.0);
            Frame::NONE
                .fill(Palette::BG_INPUT().gamma_multiply(0.4))
                .stroke(Stroke::new(1.0, Palette::BORDER()))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.label(RichText::new(clean_markdown_inline(rest)).size(12.0).color(Palette::TEXT_MUTED()));
                });
            ui.add_space(4.0);
            continue;
        }

        // 6. Unordered / ordered lists (* , - , 1. ) and task checkboxes
        if line.starts_with("* ")
            || line.starts_with("- ")
            || (line.len() > 3
                && line.chars().next().is_some_and(|c| c.is_ascii_digit())
                && line.contains(". "))
        {
            let (bullet, text, bullet_col) = if let Some(rest) = line
                .strip_prefix("- [x] ")
                .or_else(|| line.strip_prefix("- [X] "))
                .or_else(|| line.strip_prefix("* [x] "))
                .or_else(|| line.strip_prefix("* [X] "))
            {
                ("☑", rest, Palette::GREEN())
            } else if let Some(rest) = line
                .strip_prefix("- [ ] ")
                .or_else(|| line.strip_prefix("* [ ] "))
            {
                ("☐", rest, Palette::TEXT_MUTED())
            } else if let Some(rest) = line.strip_prefix("* ") {
                ("•", rest, Palette::ACCENT())
            } else if let Some(rest) = line.strip_prefix("- ") {
                ("•", rest, Palette::ACCENT())
            } else {
                let dot_idx = line.find('.').unwrap_or(1);
                let num = &line[..=dot_idx];
                let rest = line[dot_idx + 1..].trim_start();
                (num, rest, Palette::ACCENT())
            };

            ui.add_space(1.0);
            ui.horizontal_top(|ui| {
                ui.label(RichText::new(bullet).monospace().size(12.5).color(bullet_col));
                render_formatted_inline_text(ui, text);
            });
            ui.add_space(3.0);
            continue;
        }

        // 7. Normal paragraphs
        if trimmed.is_empty() {
            ui.add_space(8.0);
        } else {
            render_formatted_inline_text(ui, line);
            ui.add_space(4.0);
        }
    }

    // Flush code block if file ends inside code fence
    if in_code_block && !code_block_content.is_empty() {
        render_code_card(ui, &code_block_lang, &code_block_content);
    }
}

fn render_markdown_table(ui: &mut Ui, headers: &[&str], data_rows: &[Vec<String>]) {
    ui.add_space(6.0);
    Frame::NONE
        .fill(Palette::BG_INPUT().gamma_multiply(0.4))
        .stroke(Stroke::new(1.0, Palette::BORDER()))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            egui::Grid::new(ui.next_auto_id())
                .spacing([18.0, 7.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header row
                    for header in headers {
                        ui.label(
                            RichText::new(*header)
                                .strong()
                                .size(12.5)
                                .color(Palette::highlight_text()),
                        );
                    }
                    ui.end_row();

                    // Data rows
                    for row in data_rows {
                        for cell in row {
                            render_formatted_inline_text(ui, cell);
                        }
                        ui.end_row();
                    }
                });
        });
    ui.add_space(6.0);
}

fn render_code_card(ui: &mut Ui, lang: &str, code: &str) {
    ui.add_space(6.0);
    Frame::NONE
        .fill(Palette::BG_INPUT())
        .stroke(Stroke::new(1.0, Palette::BORDER()))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| {
            // Header bar
            if !lang.is_empty() {
                Frame::NONE
                    .fill(Palette::BG_TAB_BAR())
                    .inner_margin(Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.label(RichText::new(lang).monospace().size(11.0).color(Palette::TEXT_DIM()));
                    });
            }

            Frame::NONE
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    let lang_enum = crate::ui::syntax::Language::from_file_name(lang);
                    let job = crate::ui::syntax::highlight_code(code.trim_end(), lang_enum);
                    ui.label(job);
                });
        });
    ui.add_space(6.0);
}

/// Render inline markdown styling (bold **text**, inline `code`, [links](url), etc.)
fn render_formatted_inline_text(ui: &mut Ui, raw: &str) {
    let mut job = LayoutJob::default();
    let normal_font = FontId::proportional(12.5);
    let mono_font = FontId::monospace(11.5);

    let parts = split_inline_markdown(raw);
    for part in parts {
        match part {
            InlineToken::Text(t) => {
                job.append(
                    &t,
                    0.0,
                    TextFormat {
                        font_id: normal_font.clone(),
                        color: Palette::TEXT_MUTED(),
                        ..Default::default()
                    },
                );
            }
            InlineToken::Bold(b) => {
                job.append(
                    &b,
                    0.0,
                    TextFormat {
                        font_id: normal_font.clone(),
                        color: Palette::highlight_text(),
                        ..Default::default()
                    },
                );
            }
            InlineToken::Code(c) => {
                job.append(
                    &format!(" {c} "),
                    0.0,
                    TextFormat {
                        font_id: mono_font.clone(),
                        color: Palette::inline_code_text(),
                        background: Palette::inline_code_bg(),
                        ..Default::default()
                    },
                );
            }
            InlineToken::Link { label, .. } => {
                job.append(
                    &label,
                    0.0,
                    TextFormat {
                        font_id: normal_font.clone(),
                        color: Palette::ACCENT(),
                        underline: Stroke::new(1.0, Palette::ACCENT()),
                        ..Default::default()
                    },
                );
            }
        }
    }

    ui.add(egui::Label::new(job).wrap());
}

enum InlineToken {
    Text(String),
    Bold(String),
    Code(String),
    Link { label: String, _url: String },
}

fn split_inline_markdown(mut s: &str) -> Vec<InlineToken> {
    let mut tokens = Vec::new();

    while !s.is_empty() {
        // Find next token candidate: `**`, `` ` ``, or `[`
        let bold_pos = s.find("**");
        let code_pos = s.find('`');
        let link_pos = s.find('[');

        // Find earliest delimiter
        let mut candidates = Vec::new();
        if let Some(b) = bold_pos {
            candidates.push((b, "**"));
        }
        if let Some(c) = code_pos {
            candidates.push((c, "`"));
        }
        if let Some(l) = link_pos {
            candidates.push((l, "["));
        }

        candidates.sort_by_key(|&(pos, _)| pos);

        if let Some(&(pos, delim)) = candidates.first() {
            if pos > 0 {
                tokens.push(InlineToken::Text(s[..pos].to_string()));
            }

            if delim == "[" {
                let rest = &s[pos..];
                if let Some(close_bracket) = rest.find("](") {
                    if let Some(close_paren) = rest[close_bracket..].find(')') {
                        let label = &rest[1..close_bracket];
                        let url = &rest[close_bracket + 2..close_bracket + close_paren];
                        tokens.push(InlineToken::Link {
                            label: label.to_string(),
                            _url: url.to_string(),
                        });
                        s = &rest[close_bracket + close_paren + 1..];
                        continue;
                    }
                }
                // Not a valid link syntax, treat as text
                tokens.push(InlineToken::Text("[".to_string()));
                s = &s[pos + 1..];
            } else {
                let after_delim = &s[pos + delim.len()..];
                if let Some(end_pos) = after_delim.find(delim) {
                    let inside = &after_delim[..end_pos];
                    if delim == "**" {
                        tokens.push(InlineToken::Bold(inside.to_string()));
                    } else {
                        tokens.push(InlineToken::Code(inside.to_string()));
                    }
                    s = &after_delim[end_pos + delim.len()..];
                } else {
                    tokens.push(InlineToken::Text(s[pos..].to_string()));
                    break;
                }
            }
        } else {
            tokens.push(InlineToken::Text(s.to_string()));
            break;
        }
    }

    tokens
}

fn clean_markdown_inline(s: &str) -> String {
    s.replace("**", "")
}
