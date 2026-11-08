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

/// Render responsive table that auto-scales with available width, wraps cell contents cleanly,
/// and allows horizontal scrolling when columns reach their minimum readable width.
fn render_markdown_table(ui: &mut Ui, headers: &[&str], data_rows: &[Vec<String>]) {
    ui.add_space(6.0);

    let num_cols = headers.len().max(1);
    let avail_width = (ui.available_width() - 24.0).max(100.0);

    // Compute relative weights based on maximum content length in each column
    let mut max_lens = vec![4usize; num_cols];
    for (i, h) in headers.iter().enumerate() {
        if i < num_cols {
            max_lens[i] = max_lens[i].max(h.chars().count().max(3));
        }
    }
    for row in data_rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                max_lens[i] = max_lens[i].max(cell.chars().count().min(35));
            }
        }
    }

    let sum_lens: usize = max_lens.iter().sum();
    let sum_lens = sum_lens.max(1) as f32;

    // Distribute widths proportionally with a minimum column width of 75.0px
    let min_col_w = 75.0;
    let mut col_widths: Vec<f32> = Vec::with_capacity(num_cols);
    for &len in &max_lens {
        let proportion = (len as f32) / sum_lens;
        let w = (proportion * avail_width).max(min_col_w);
        col_widths.push(w);
    }

    let min_total_table_w: f32 = col_widths.iter().sum::<f32>() + ((num_cols as f32) * 14.0);

    egui::ScrollArea::horizontal()
        .id_salt(ui.next_auto_id())
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(min_total_table_w.max(avail_width));

            Frame::NONE
                .fill(Palette::BG_INPUT().gamma_multiply(0.45))
                .stroke(Stroke::new(1.0, Palette::BORDER()))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    egui::Grid::new(ui.next_auto_id())
                        .spacing([12.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Header row
                            for (i, header) in headers.iter().enumerate() {
                                let col_w = col_widths.get(i).copied().unwrap_or(min_col_w);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(col_w, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_width(col_w);
                                        ui.label(
                                            RichText::new(*header)
                                                .strong()
                                                .size(12.5)
                                                .color(Palette::highlight_text()),
                                        );
                                    },
                                );
                            }
                            ui.end_row();

                            // Data rows
                            for row in data_rows {
                                for (i, cell) in row.iter().enumerate() {
                                    let col_w = col_widths.get(i).copied().unwrap_or(min_col_w);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(col_w, 0.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            ui.set_width(col_w);
                                            render_formatted_inline_text(ui, cell);
                                        },
                                    );
                                }
                                ui.end_row();
                            }
                        });
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

/// Render inline markdown styling (bold **text**, inline `code`, [links](url), math expressions $...$, etc.)
fn render_formatted_inline_text(ui: &mut Ui, raw: &str) {
    let mut job = LayoutJob::default();
    let normal_font = FontId::proportional(12.5);
    let mono_font = FontId::monospace(11.5);

    let parts = split_inline_markdown(raw);
    for part in parts {
        match part {
            InlineToken::Text(t) => {
                let formatted = format_latex_math(&t);
                job.append(
                    &formatted,
                    0.0,
                    TextFormat {
                        font_id: normal_font.clone(),
                        color: Palette::TEXT_MUTED(),
                        ..Default::default()
                    },
                );
            }
            InlineToken::Bold(b) => {
                let formatted = format_latex_math(&b);
                job.append(
                    &formatted,
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
            InlineToken::Math(m) => {
                job.append(
                    &m,
                    0.0,
                    TextFormat {
                        font_id: normal_font.clone(),
                        color: Palette::ACCENT(),
                        italics: true,
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
    Math(String),
    Link { label: String, _url: String },
}

fn split_inline_markdown(mut s: &str) -> Vec<InlineToken> {
    let mut tokens = Vec::new();

    while !s.is_empty() {
        // Find next token candidate: `**`, `` ` ``, `[`, or `$`
        let bold_pos = s.find("**");
        let code_pos = s.find('`');
        let link_pos = s.find('[');
        let math_pos = s.find('$');

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
        if let Some(m) = math_pos {
            candidates.push((m, "$"));
        }

        candidates.sort_by_key(|&(pos, _)| pos);

        if let Some(&(pos, delim)) = candidates.first() {
            if pos > 0 {
                tokens.push(InlineToken::Text(s[..pos].to_string()));
            }

            if delim == "$" {
                let rest = &s[pos..];
                if rest.starts_with("$$") {
                    let after_open = &rest[2..];
                    if let Some(close_pos) = after_open.find("$$") {
                        let formula = &after_open[..close_pos];
                        tokens.push(InlineToken::Math(format_latex_math(formula)));
                        s = &after_open[close_pos + 2..];
                        continue;
                    }
                } else {
                    let after_open = &rest[1..];
                    if let Some(close_pos) = after_open.find('$') {
                        let formula = &after_open[..close_pos];
                        tokens.push(InlineToken::Math(format_latex_math(formula)));
                        s = &after_open[close_pos + 1..];
                        continue;
                    }
                }
                // Unclosed or standalone dollar sign, treat as text
                tokens.push(InlineToken::Text("$".to_string()));
                s = &s[pos + 1..];
            } else if delim == "[" {
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
    let unbold = s.replace("**", "");
    format_latex_math(&unbold)
}

/// Convert LaTeX math notation (e.g. \boldsymbol{\Sigma}_v, v_x, \sigma_x^2, \rho, etc.)
/// into clean, readable Unicode math symbols.
pub fn format_latex_math(raw: &str) -> String {
    let mut s = raw.to_string();

    // 1. Strip LaTeX font and weight wrappers: \boldsymbol{...}, \mathbf{...}, etc.
    let wrappers = [
        "\\boldsymbol{", "\\mathbf{", "\\mathit{", "\\mathrm{", "\\text{", "\\bm{",
    ];
    for w in wrappers {
        while let Some(pos) = s.find(w) {
            let start = pos + w.len();
            let mut depth = 1;
            let mut end = None;
            for (idx, c) in s[start..].char_indices() {
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + idx);
                        break;
                    }
                }
            }
            if let Some(e) = end {
                let inside = &s[start..e];
                let prefix = &s[..pos];
                let suffix = &s[e + 1..];
                s = format!("{prefix}{inside}{suffix}");
            } else {
                s.replace_range(pos..pos + w.len(), "");
            }
        }
    }

    // 2. Greek letters (ordered from longer to shorter names to avoid prefix conflicts)
    let greek = [
        ("\\varepsilon", "ε"), ("\\vartheta", "θ"), ("\\varrho", "ρ"), ("\\varphi", "φ"),
        ("\\alpha", "α"), ("\\beta", "β"), ("\\gamma", "γ"), ("\\Gamma", "Γ"),
        ("\\delta", "δ"), ("\\Delta", "Δ"), ("\\epsilon", "ε"), ("\\zeta", "ζ"),
        ("\\eta", "η"), ("\\theta", "θ"), ("\\Theta", "Θ"), ("\\iota", "ι"),
        ("\\kappa", "κ"), ("\\lambda", "λ"), ("\\Lambda", "Λ"), ("\\mu", "μ"),
        ("\\nu", "ν"), ("\\xi", "ξ"), ("\\Xi", "Ξ"), ("\\pi", "π"),
        ("\\Pi", "Π"), ("\\rho", "ρ"), ("\\sigma", "σ"), ("\\Sigma", "Σ"),
        ("\\tau", "τ"), ("\\upsilon", "υ"), ("\\phi", "φ"), ("\\Phi", "Φ"),
        ("\\chi", "χ"), ("\\psi", "ψ"), ("\\Psi", "Ψ"), ("\\omega", "ω"),
        ("\\Omega", "Ω"),
    ];
    for (tex, uni) in greek {
        s = s.replace(tex, uni);
    }

    // 3. Mathematical operators & relations
    let symbols = [
        ("\\times", "×"), ("\\cdot", "·"), ("\\pm", "±"), ("\\mp", "∓"),
        ("\\leq", "≤"), ("\\le", "≤"), ("\\geq", "≥"), ("\\ge", "≥"),
        ("\\neq", "≠"), ("\\ne", "≠"), ("\\approx", "≈"), ("\\equiv", "≡"),
        ("\\in", "∈"), ("\\notin", "∉"), ("\\subset", "⊂"), ("\\supset", "⊃"),
        ("\\infty", "∞"), ("\\rightarrow", "→"), ("\\to", "→"), ("\\leftarrow", "←"),
        ("\\partial", "∂"), ("\\nabla", "∇"), ("\\sum", "∑"), ("\\prod", "∏"),
        ("\\int", "∫"), ("\\forall", "∀"), ("\\exists", "∃"),
    ];
    for (tex, uni) in symbols {
        s = s.replace(tex, uni);
    }

    // 4. Superscripts (^2, ^3, ^1, ^0, ^T, ^-1, ^n, etc.)
    let supers = [
        ("^0", "⁰"), ("^1", "¹"), ("^2", "²"), ("^3", "³"), ("^4", "⁴"),
        ("^5", "⁵"), ("^6", "⁶"), ("^7", "⁷"), ("^8", "⁸"), ("^9", "⁹"),
        ("^{0}", "⁰"), ("^{1}", "¹"), ("^{2}", "²"), ("^{3}", "³"), ("^{4}", "⁴"),
        ("^{5}", "⁵"), ("^{6}", "⁶"), ("^{7}", "⁷"), ("^{8}", "⁸"), ("^{9}", "⁹"),
        ("^T", "ᵀ"), ("^{T}", "ᵀ"), ("^-1", "⁻¹"), ("^{-1}", "⁻¹"), ("^n", "ⁿ"), ("^{n}", "ⁿ"),
    ];
    for (tex, uni) in supers {
        s = s.replace(tex, uni);
    }

    // 5. Subscripts (_0.._9, _x, _y, _z, _v, _i, _j, _k, _n, _m, _t)
    let subs = [
        ("_0", "₀"), ("_1", "₁"), ("_2", "₂"), ("_3", "₃"), ("_4", "₄"),
        ("_5", "₅"), ("_6", "₆"), ("_7", "₇"), ("_8", "₈"), ("_9", "₉"),
        ("_{0}", "₀"), ("_{1}", "₁"), ("_{2}", "₂"), ("_{3}", "₃"), ("_{4}", "₄"),
        ("_{5}", "₅"), ("_{6}", "₆"), ("_{7}", "₇"), ("_{8}", "₈"), ("_{9}", "₉"),
        ("_x", "ₓ"), ("_{x}", "ₓ"), ("_y", "ᵧ"), ("_{y}", "ᵧ"),
        ("_v", "ᵥ"), ("_{v}", "ᵥ"), ("_i", "ᵢ"), ("_{i}", "ᵢ"),
        ("_j", "ⱼ"), ("_{j}", "ⱼ"), ("_k", "ₖ"), ("_{k}", "ₖ"),
        ("_n", "ₙ"), ("_{n}", "ₙ"), ("_m", "ₘ"), ("_{m}", "ₘ"),
        ("_t", "ₜ"), ("_{t}", "ₜ"),
    ];
    for (tex, uni) in subs {
        s = s.replace(tex, uni);
    }

    // 6. Clean braces
    s = s.replace('{', "").replace('}', "");

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_latex_math() {
        let input1 = r"\boldsymbol{\Sigma}_v";
        assert_eq!(format_latex_math(input1), "Σᵥ");

        let input2 = r"v_x, v_y, \sigma_x^2, \sigma_y^2, \rho";
        assert_eq!(format_latex_math(input2), "vₓ, vᵧ, σₓ², σᵧ², ρ");

        let input3 = r"v_z";
        assert_eq!(format_latex_math(input3), "v_z");
    }
}

