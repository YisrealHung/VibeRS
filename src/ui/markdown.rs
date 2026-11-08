use crate::theme::Palette;
use egui::{text::LayoutJob, Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, TextFormat, Ui};

/// Render beautiful Markdown blocks like Antigravity IDE
pub fn render_markdown_message(ui: &mut Ui, markdown: &str) {
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();

    for line in markdown.lines() {
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
            ui.label(RichText::new(clean_markdown_inline(rest)).size(15.5).strong().color(Color32::WHITE));
            ui.add_space(5.0);
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            ui.add_space(14.0);
            ui.label(RichText::new(clean_markdown_inline(rest)).size(17.0).strong().color(Palette::ACCENT()));
            ui.add_space(6.0);
            continue;
        }

        // 4. Blockquotes (> ...)
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

        // 5. Unordered / ordered lists (* , - , 1. )
        if line.starts_with("* ") || line.starts_with("- ") || (line.len() > 3 && line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". ")) {
            let (bullet, text) = if let Some(rest) = line.strip_prefix("* ") {
                ("•", rest)
            } else if let Some(rest) = line.strip_prefix("- ") {
                ("•", rest)
            } else {
                let dot_idx = line.find('.').unwrap_or(1);
                let num = &line[..=dot_idx];
                let rest = line[dot_idx + 1..].trim_start();
                (num, rest)
            };

            ui.add_space(1.0);
            ui.horizontal_top(|ui| {
                ui.label(RichText::new(bullet).monospace().size(12.0).color(Palette::ACCENT()));
                render_formatted_inline_text(ui, text);
            });
            ui.add_space(3.0);
            continue;
        }

        // 6. Normal paragraphs
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

fn render_code_card(ui: &mut Ui, lang: &str, code: &str) {
    ui.add_space(6.0);
    Frame::NONE
        .fill(Palette::BG_EDITOR())
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
                    ui.label(
                        RichText::new(code.trim_end())
                            .font(FontId::monospace(12.0))
                            .color(Palette::TEXT_PRIMARY()),
                    );
                });
        });
    ui.add_space(6.0);
}

/// Render inline markdown styling (bold **text**, inline `code`, etc.)
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
                        color: Color32::WHITE,
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
                        color: Palette::ACCENT(),
                        background: Palette::BG_INPUT(),
                        ..Default::default()
                    },
                );
            }
        }
    }

    ui.add(egui::Label::new(job).wrap());
}

#[derive(Debug)]
enum InlineToken {
    Text(String),
    Bold(String),
    Code(String),
}

fn split_inline_markdown(mut s: &str) -> Vec<InlineToken> {
    let mut tokens = Vec::new();

    while !s.is_empty() {
        let bold_pos = s.find("**");
        let code_pos = s.find('`');

        let next_delim = match (bold_pos, code_pos) {
            (Some(b), Some(c)) => {
                if b <= c {
                    Some((b, "**"))
                } else {
                    Some((c, "`"))
                }
            }
            (Some(b), None) => Some((b, "**")),
            (None, Some(c)) => Some((c, "`")),
            (None, None) => None,
        };

        if let Some((pos, delim)) = next_delim {
            if pos > 0 {
                tokens.push(InlineToken::Text(s[..pos].to_string()));
            }
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

