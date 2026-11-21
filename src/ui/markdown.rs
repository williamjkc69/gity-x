use std::collections::HashMap;

use eframe::egui::{
    self,
    text::{LayoutJob, TextFormat},
    Color32, FontId, Margin, RichText, Rounding, Stroke,
};

use crate::theme;

/// Strictly sorted list of known HTML tags so `binary_search` is exact and
/// programming generics like `Result<User>`, `Option<T>`, `Vec<String>` are never stripped.
const KNOWN_HTML_TAGS: &[&str] = &[
    "a", "abbr", "article", "aside", "b", "bdi", "bdo", "blockquote", "br", "caption", "center",
    "cite", "code", "col", "colgroup", "dd", "del", "details", "dfn", "div", "dl", "dt", "em",
    "figcaption", "figure", "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "i",
    "img", "input", "ins", "kbd", "li", "main", "mark", "nav", "ol", "p", "picture", "pre", "q",
    "rp", "rt", "ruby", "s", "samp", "script", "section", "small", "source", "span", "strike",
    "strong", "style", "sub", "summary", "sup", "table", "tbody", "td", "tfoot", "th", "thead",
    "time", "tr", "tt", "u", "ul", "var", "video", "wbr",
];

fn is_known_html_tag(tag_lower: &str) -> bool {
    KNOWN_HTML_TAGS.binary_search(&tag_lower).is_ok()
}

/// Renders Markdown + embedded GitHub/GitLab/Bitbucket HTML safely into an egui UI.
pub fn render(ui: &mut egui::Ui, raw: &str, base_size: f32) {
    let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
    let cleaned = strip_comments_and_nonvisual(&normalized);
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return;
    }
    let refs = collect_reference_links(trimmed);
    render_depth(ui, trimmed, base_size, 0, &refs);
}

enum BodySegment {
    Markup(String),
    Details {
        summary: String,
        body: String,
        open: bool,
    },
}

fn render_depth(
    ui: &mut egui::Ui,
    body: &str,
    base_size: f32,
    depth: usize,
    refs: &HashMap<String, String>,
) {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return;
    }

    if depth < 5 && trimmed.to_ascii_lowercase().contains("<details") {
        let segments = split_details_segments(trimmed);
        for (seg_idx, seg) in segments.into_iter().enumerate() {
            match seg {
                BodySegment::Markup(chunk) => {
                    render_markup_block_depth(ui, &chunk, base_size, refs, depth);
                }
                BodySegment::Details {
                    summary,
                    body: inner,
                    open,
                } => {
                    let sum_clean = decode_html_entities(&strip_known_html_tags(&summary))
                        .replace('\n', " ")
                        .trim()
                        .to_string();
                    let title = if sum_clean.is_empty() {
                        "Details".to_string()
                    } else {
                        sum_clean
                    };
                    egui::Frame::none()
                        .fill(theme::BG_SURFACE)
                        .rounding(Rounding::same(6.0))
                        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                        .inner_margin(Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            egui::CollapsingHeader::new(
                                RichText::new(&title)
                                    .size(base_size)
                                    .strong()
                                    .color(theme::ACCENT_CYAN),
                            )
                            .id_salt(("md_details", depth, seg_idx, &title))
                            .default_open(open)
                            .show(ui, |ui| {
                                ui.add_space(4.0);
                                render_depth(ui, &inner, base_size, depth + 1, refs);
                            });
                        });
                    ui.add_space(5.0);
                }
            }
        }
        return;
    }

    render_markup_block_depth(ui, trimmed, base_size, refs, depth);
}

fn render_markup_block_depth(
    ui: &mut egui::Ui,
    raw: &str,
    base_size: f32,
    refs: &HashMap<String, String>,
    depth: usize,
) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }

    let converted = if looks_like_html(trimmed) {
        html_to_markdown_preserving_fences(trimmed)
    } else {
        decode_entities_outside_code(trimmed)
    };
    let resolved = if refs.is_empty() {
        converted
    } else {
        resolve_reference_links(&converted, refs)
    };
    let body = resolved.trim();
    if body.is_empty() {
        return;
    }

    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();

        // Blank line spacing
        if t.is_empty() {
            ui.add_space(5.0);
            i += 1;
            continue;
        }

        // Hidden Markdown reference comments or reference-link definitions
        if is_hidden_md_comment(t) || parse_reference_def(t).is_some() {
            i += 1;
            continue;
        }

        // Fenced code block (``` or ~~~, 3+ chars)
        if let Some((fence_ch, fence_len, lang)) = parse_fence_open(t) {
            i += 1;
            let mut code = String::new();
            while i < lines.len() && !is_fence_close(lines[i].trim(), fence_ch, fence_len) {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            egui::Frame::none()
                .fill(theme::BG_SURFACE)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                .inner_margin(Margin::symmetric(10.0, 7.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if !lang.is_empty() {
                        ui.label(
                            RichText::new(lang)
                                .monospace()
                                .size(10.0)
                                .color(theme::TEXT_MUTED),
                        );
                        ui.add_space(2.0);
                    }
                    egui::ScrollArea::horizontal()
                        .id_salt(("md_code", depth, i, code.len()))
                        .show(ui, |ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                            ui.label(
                                RichText::new(&code)
                                    .font(FontId::monospace(11.5))
                                    .color(theme::TEXT_PRIMARY),
                            );
                        });
                });
            ui.add_space(5.0);
            continue;
        }

        // Setext heading: plain paragraph line followed by === or ---
        if i + 1 < lines.len() && is_setext_heading_candidate(t) {
            if let Some(level) = parse_setext_underline(lines[i + 1].trim()) {
                render_heading(ui, level, t, base_size);
                i += 2;
                continue;
            }
        }

        // Horizontal rule (---, ***, ___, 3+ chars with optional spaces)
        if is_horizontal_rule(t) {
            ui.add_space(3.0);
            ui.separator();
            ui.add_space(3.0);
            i += 1;
            continue;
        }

        // ATX Headings (# .. ######)
        if let Some((level, heading)) = parse_heading(t) {
            render_heading(ui, level, heading, base_size);
            i += 1;
            continue;
        }

        // Markdown table block (header row followed by separator row --- | ---)
        if is_table_start(&lines, i) {
            let mut rows: Vec<Vec<String>> = Vec::new();
            while i < lines.len() {
                let row_t = lines[i].trim();
                if row_t.is_empty() || !row_t.contains('|') {
                    break;
                }
                let cells = split_table_row(row_t);
                if is_table_separator_row(&cells) {
                    i += 1;
                    continue;
                }
                if !cells.is_empty() {
                    rows.push(cells);
                }
                i += 1;
            }
            if !rows.is_empty() {
                egui::Frame::none()
                    .fill(theme::BG_SURFACE)
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::horizontal()
                            .id_salt(("md_table_scroll", depth, i, rows.len()))
                            .show(ui, |ui| {
                                egui::Grid::new(("md_table", depth, i, rows.len()))
                                    .striped(true)
                                    .spacing([14.0, 4.0])
                                    .show(ui, |ui| {
                                        for (r_idx, row) in rows.iter().enumerate() {
                                            for cell in row {
                                                if r_idx == 0 {
                                                    render_inline_md_styled(
                                                        ui,
                                                        cell,
                                                        base_size - 0.5,
                                                        Color32::WHITE,
                                                        true,
                                                    );
                                                } else {
                                                    render_inline_md(ui, cell, base_size - 0.5);
                                                }
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                ui.add_space(5.0);
            }
            continue;
        }

        // Blockquote (> ...) & GitHub Alerts (> [!NOTE] / [!TIP] / [!IMPORTANT] / [!WARNING] / [!CAUTION])
        if let Some(quoted) = strip_blockquote_prefix(t) {
            let mut quote_lines = vec![quoted];
            i += 1;
            while i < lines.len() {
                if let Some(q) = strip_blockquote_prefix(lines[i].trim()) {
                    quote_lines.push(q);
                    i += 1;
                } else {
                    break;
                }
            }
            let (alert_meta, body_start_idx) = parse_github_alert(quote_lines[0])
                .map(|m| (Some(m), 1))
                .unwrap_or((None, 0));

            let (border_col, fill_col) = match alert_meta {
                Some((_, col)) => (col, col.linear_multiply(0.12)),
                None => (theme::ACCENT_CYAN.linear_multiply(0.55), theme::BG_SURFACE),
            };

            egui::Frame::none()
                .fill(fill_col)
                .rounding(Rounding::same(5.0))
                .stroke(Stroke::new(1.0_f32, border_col))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if let Some((label, col)) = alert_meta {
                        ui.label(RichText::new(label).size(base_size - 0.5).strong().color(col));
                        ui.add_space(2.0);
                    }
                    let inner_body = quote_lines[body_start_idx..].join("\n");
                    if !inner_body.trim().is_empty() {
                        if depth < 5 {
                            render_markup_block_depth(
                                ui,
                                &inner_body,
                                base_size - 0.5,
                                refs,
                                depth + 1,
                            );
                        } else {
                            for ql in &quote_lines[body_start_idx..] {
                                if ql.is_empty() {
                                    ui.add_space(3.0);
                                } else {
                                    render_inline_md(ui, ql, base_size - 0.5);
                                }
                            }
                        }
                    }
                });
            ui.add_space(4.0);
            continue;
        }

        // Indentation for list items
        let indent_spaces = line.len() - line.trim_start().len();
        let indent_px = ((indent_spaces / 2) as f32 * 14.0).min(42.0);

        // Task list (- [x] / - [ ])
        if let Some((checked, rest)) = parse_task_item(t) {
            ui.horizontal_top(|ui| {
                ui.add_space(indent_px);
                if checked {
                    ui.label(
                        RichText::new("✓")
                            .size(base_size)
                            .strong()
                            .color(theme::ACCENT_GREEN),
                    );
                } else {
                    ui.label(RichText::new("□").size(base_size).color(theme::TEXT_MUTED));
                }
                render_inline_md(ui, rest, base_size);
            });
            i += 1;
            continue;
        }

        // Unordered bullet list (- / * / +)
        if let Some(rest) = t
            .strip_prefix("- ")
            .or_else(|| t.strip_prefix("* "))
            .or_else(|| t.strip_prefix("+ "))
        {
            ui.horizontal_top(|ui| {
                ui.add_space(indent_px + 4.0);
                ui.label(RichText::new("•").size(base_size).color(theme::ACCENT_CYAN));
                render_inline_md(ui, rest, base_size);
            });
            i += 1;
            continue;
        }

        // Ordered numbered list (1. ... or 1) ...)
        if let Some((num_lbl, rest)) = parse_ordered_list_prefix(t) {
            ui.horizontal_top(|ui| {
                ui.add_space(indent_px + 4.0);
                ui.label(
                    RichText::new(num_lbl)
                        .size(base_size)
                        .color(theme::ACCENT_CYAN),
                );
                render_inline_md(ui, rest, base_size);
            });
            i += 1;
            continue;
        }

        // Normal paragraph line with inline markdown
        render_inline_md(ui, t, base_size);
        i += 1;
    }
}

fn is_setext_heading_candidate(trimmed: &str) -> bool {
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && !trimmed.starts_with('>')
        && !trimmed.starts_with('|')
        && !trimmed.starts_with("- ")
        && !trimmed.starts_with("* ")
        && !trimmed.starts_with("+ ")
        && !is_horizontal_rule(trimmed)
        && parse_ordered_list_prefix(trimmed).is_none()
        && parse_fence_open(trimmed).is_none()
}

fn strip_optional_closing_hashes(heading: &str) -> &str {
    let trimmed = heading.trim_end();
    let without_hashes = trimmed.trim_end_matches('#');
    if without_hashes.len() < trimmed.len()
        && (without_hashes.is_empty()
            || without_hashes.ends_with(' ')
            || without_hashes.ends_with('\t'))
    {
        without_hashes.trim_end()
    } else {
        trimmed
    }
}

fn render_heading(ui: &mut egui::Ui, level: usize, text: &str, base_size: f32) {
    let (size, color) = match level {
        1 => (base_size + 4.5, Color32::WHITE),
        2 => (base_size + 2.5, Color32::WHITE),
        3 => (base_size + 1.0, theme::ACCENT_CYAN),
        _ => (base_size + 0.5, theme::TEXT_PRIMARY),
    };
    ui.add_space(4.0);
    let clean_heading = strip_optional_closing_hashes(text);
    render_inline_md_styled(ui, clean_heading, size, color, true);
    if level <= 2 {
        ui.separator();
    }
    ui.add_space(2.0);
}

fn parse_fence_open(trimmed: &str) -> Option<(u8, usize, &str)> {
    let first = *trimmed.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let count = trimmed.bytes().take_while(|&b| b == first).count();
    if count < 3 {
        return None;
    }
    let rest = trimmed[count..].trim();
    // Inline code like ```foo``` on a single line is NOT a fenced block
    if first == b'`' && rest.contains('`') {
        return None;
    }
    Some((first, count, rest))
}

fn is_fence_close(trimmed: &str, fence_ch: u8, min_len: usize) -> bool {
    let count = trimmed.bytes().take_while(|&b| b == fence_ch).count();
    count >= min_len && trimmed[count..].trim().is_empty()
}

fn parse_setext_underline(trimmed: &str) -> Option<usize> {
    if trimmed.len() < 3 {
        return None;
    }
    if trimmed.bytes().all(|b| b == b'=') {
        Some(1)
    } else if trimmed.bytes().all(|b| b == b'-') {
        Some(2)
    } else {
        None
    }
}

fn is_horizontal_rule(trimmed: &str) -> bool {
    let non_space: Vec<u8> = trimmed.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if non_space.len() < 3 {
        return false;
    }
    let ch = non_space[0];
    matches!(ch, b'-' | b'*' | b'_') && non_space.iter().all(|&b| b == ch)
}

fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if (1..=6).contains(&hashes) {
        let rest = line[hashes..].strip_prefix(' ')?;
        Some((hashes, rest.trim()))
    } else {
        None
    }
}

fn is_table_start(lines: &[&str], idx: usize) -> bool {
    let t = lines[idx].trim();
    if !t.contains('|') || idx + 1 >= lines.len() {
        return false;
    }
    let next = lines[idx + 1].trim();
    if !next.contains('|') && !next.contains('-') {
        return false;
    }
    let next_cells = split_table_row(next);
    is_table_separator_row(&next_cells)
}

/// Splits a Markdown table row on `|` while respecting escaped `\|` and inline backticks `` `a | b` ``.
fn split_table_row(row_t: &str) -> Vec<String> {
    let inner = row_t.strip_prefix('|').unwrap_or(row_t);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let bytes = inner.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'|' {
            cur.push('|');
            i += 2;
            continue;
        }
        if bytes[i] == b'`' {
            let ticks = inner[i..].bytes().take_while(|&b| b == b'`').count();
            let marker = &inner[i..i + ticks];
            if let Some(rel_end) = inner[i + ticks..].find(marker) {
                let end = i + ticks + rel_end + ticks;
                cur.push_str(&inner[i..end]);
                i = end;
                continue;
            }
        }
        if bytes[i] == b'|' {
            cells.push(cur.trim().to_string());
            cur.clear();
            i += 1;
            continue;
        }
        let ch_len = inner[i..].chars().next().map_or(1, char::len_utf8);
        cur.push_str(&inner[i..i + ch_len]);
        i += ch_len;
    }
    cells.push(cur.trim().to_string());
    cells
}

fn is_table_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|c| {
            !c.is_empty()
                && c.contains('-')
                && c.chars().all(|ch| matches!(ch, '-' | ':' | ' '))
        })
}

fn strip_blockquote_prefix(trimmed: &str) -> Option<&str> {
    trimmed
        .strip_prefix("> ")
        .or_else(|| trimmed.strip_prefix('>'))
        .map(str::trim_start)
}

fn parse_github_alert(first_line: &str) -> Option<(&'static str, Color32)> {
    match first_line.trim().to_ascii_uppercase().as_str() {
        "[!NOTE]" => Some(("NOTE", theme::ACCENT_CYAN)),
        "[!TIP]" => Some(("TIP", theme::ACCENT_GREEN)),
        "[!IMPORTANT]" => Some(("IMPORTANT", theme::ACCENT_PURPLE)),
        "[!WARNING]" => Some(("WARNING", theme::ACCENT_AMBER)),
        "[!CAUTION]" => Some(("CAUTION", theme::ACCENT_ROSE)),
        _ => None,
    }
}

fn parse_task_item(trimmed: &str) -> Option<(bool, &str)> {
    for prefix in ["- [x] ", "- [X] ", "* [x] ", "* [X] ", "+ [x] ", "+ [X] "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some((true, rest));
        }
    }
    for prefix in ["- [ ] ", "* [ ] ", "+ [ ] "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return Some((false, rest));
        }
    }
    None
}

fn parse_ordered_list_prefix(trimmed: &str) -> Option<(&str, &str)> {
    let digits = trimmed.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 || digits > 4 {
        return None;
    }
    let after = &trimmed[digits..];
    let rest = after.strip_prefix(". ").or_else(|| after.strip_prefix(") "))?;
    Some((&trimmed[..digits + 1], rest))
}

fn is_hidden_md_comment(trimmed: &str) -> bool {
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("[//]: #")
        || lower.starts_with("[//]: <>")
        || lower.starts_with("[comment]: #")
        || lower.starts_with("[comment]: <>")
}

/// Parses `[id]: https://example.com` reference link definitions.
fn parse_reference_def(trimmed: &str) -> Option<(String, String)> {
    let rest = trimmed.strip_prefix('[')?;
    let end_id = rest.find("]:")?;
    let id = rest[..end_id].trim();
    if id.is_empty() || id == "//" || id.eq_ignore_ascii_case("comment") {
        return None;
    }
    let after = rest[end_id + 2..].trim();
    let url_token = after
        .split_whitespace()
        .next()?
        .trim_matches(|c| c == '<' || c == '>');
    if is_openable_url(url_token) {
        Some((id.to_ascii_lowercase(), url_token.to_string()))
    } else {
        None
    }
}

fn is_openable_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:")
}

fn collect_reference_links(input: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut in_fence = false;
    let mut fence_ch = b'`';
    let mut fence_len = 3;
    for line in input.lines() {
        let t = line.trim();
        if !in_fence {
            if let Some((ch, len, _)) = parse_fence_open(t) {
                in_fence = true;
                fence_ch = ch;
                fence_len = len;
                continue;
            }
            if let Some((id, url)) = parse_reference_def(t) {
                map.insert(id, url);
            }
        } else if is_fence_close(t, fence_ch, fence_len) {
            in_fence = false;
        }
    }
    map
}

fn resolve_reference_links(input: &str, refs: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_fence = false;
    let mut fence_ch = b'`';
    let mut fence_len = 3;

    for line in input.split_inclusive('\n') {
        let t = line.trim();
        if !in_fence {
            if let Some((ch, len, _)) = parse_fence_open(t) {
                in_fence = true;
                fence_ch = ch;
                fence_len = len;
                out.push_str(line);
                continue;
            }
            if parse_reference_def(t).is_some() {
                continue;
            }
            out.push_str(&resolve_refs_in_line(line, refs));
        } else {
            out.push_str(line);
            if is_fence_close(t, fence_ch, fence_len) {
                in_fence = false;
            }
        }
    }
    out
}

fn resolve_refs_in_line(line: &str, refs: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let ticks = line[i..].bytes().take_while(|&b| b == b'`').count();
            let marker = &line[i..i + ticks];
            if let Some(end_rel) = line[i + ticks..].find(marker) {
                let end = i + ticks + end_rel + ticks;
                out.push_str(&line[i..end]);
                i = end;
                continue;
            }
        }
        if bytes[i] == b'[' {
            if let Some(mid_rel) = line[i + 1..].find("][") {
                let label = &line[i + 1..i + 1 + mid_rel];
                let ref_start = i + 1 + mid_rel + 2;
                if let Some(ref_end_rel) = line[ref_start..].find(']') {
                    let ref_id = &line[ref_start..ref_start + ref_end_rel];
                    let key = if ref_id.trim().is_empty() {
                        label.trim().to_ascii_lowercase()
                    } else {
                        ref_id.trim().to_ascii_lowercase()
                    };
                    if let Some(url) = refs.get(&key) {
                        out.push_str(&format!("[{label}]({url})"));
                        i = ref_start + ref_end_rel + 1;
                        continue;
                    }
                }
            }
        }
        let ch_len = line[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&line[i..i + ch_len]);
        i += ch_len;
    }
    out
}

#[derive(Clone, Copy)]
struct InlineStyle {
    size: f32,
    base_color: Color32,
    bold: bool,
    italic: bool,
    strike: bool,
    link: bool,
}

impl InlineStyle {
    fn to_text_format(self) -> TextFormat {
        let color = if self.link {
            theme::ACCENT_CYAN
        } else if self.strike {
            theme::TEXT_MUTED
        } else if self.bold {
            Color32::WHITE
        } else if self.italic {
            theme::TEXT_SECONDARY
        } else {
            self.base_color
        };

        TextFormat {
            font_id: FontId::proportional(self.size),
            color,
            italics: self.italic,
            strikethrough: if self.strike {
                Stroke::new(1.0_f32, theme::TEXT_MUTED)
            } else {
                Stroke::NONE
            },
            underline: if self.link {
                Stroke::new(1.0_f32, theme::ACCENT_CYAN)
            } else {
                Stroke::NONE
            },
            ..Default::default()
        }
    }

    fn code_format(self) -> TextFormat {
        TextFormat {
            font_id: FontId::monospace((self.size - 0.5).max(10.5)),
            color: theme::ACCENT_CYAN,
            background: theme::BG_SURFACE,
            underline: if self.link {
                Stroke::new(1.0_f32, theme::ACCENT_CYAN)
            } else {
                Stroke::NONE
            },
            strikethrough: if self.strike {
                Stroke::new(1.0_f32, theme::TEXT_MUTED)
            } else {
                Stroke::NONE
            },
            ..Default::default()
        }
    }
}

fn render_inline_md(ui: &mut egui::Ui, text: &str, base_size: f32) {
    render_inline_md_styled(ui, text, base_size, theme::TEXT_PRIMARY, false);
}

fn render_inline_md_styled(
    ui: &mut egui::Ui,
    text: &str,
    base_size: f32,
    base_color: Color32,
    bold: bool,
) {
    let mut job = LayoutJob::default();
    let mut links: Vec<(String, String)> = Vec::new();
    let style = InlineStyle {
        size: base_size,
        base_color,
        bold,
        italic: false,
        strike: false,
        link: false,
    };
    append_inline_spans(text, style, 0, &mut job, &mut links);

    let resp = ui.label(job);
    if links.len() == 1 {
        let url = &links[0].1;
        if resp
            .on_hover_text(url)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            let _ = webbrowser::open(url);
        }
    } else if links.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            for (lbl, url) in links {
                if ui
                    .small_button(RichText::new(format!("→ {lbl}")).color(theme::ACCENT_CYAN))
                    .on_hover_text(&url)
                    .clicked()
                {
                    let _ = webbrowser::open(&url);
                }
            }
        });
    }
}

fn append_inline_spans(
    text: &str,
    style: InlineStyle,
    depth: usize,
    job: &mut LayoutJob,
    links: &mut Vec<(String, String)>,
) {
    if depth > 6 {
        job.append(text, 0.0, style.to_text_format());
        return;
    }

    let bytes = text.as_bytes();
    let mut i = 0;
    let mut plain_start = 0;
    let fmt = style.to_text_format();
    let code_fmt = style.code_format();

    let flush_plain = |job: &mut LayoutJob, start: usize, end: usize| {
        if end > start {
            job.append(&text[start..end], 0.0, fmt.clone());
        }
    };

    while i < bytes.len() {
        // Backslash escape (\*, \_, \[, \], \`, \\, \#, \|, etc.)
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if matches!(
                next,
                b'\\'
                    | b'`'
                    | b'*'
                    | b'_'
                    | b'['
                    | b']'
                    | b'('
                    | b')'
                    | b'#'
                    | b'~'
                    | b'<'
                    | b'>'
                    | b'|'
            ) {
                flush_plain(job, plain_start, i);
                job.append(&text[i + 1..i + 2], 0.0, fmt.clone());
                i += 2;
                plain_start = i;
                continue;
            }
        }

        // Inline code `...` or ``...`` (multi-backtick safe)
        if bytes[i] == b'`' {
            let ticks = text[i..].bytes().take_while(|&b| b == b'`').count();
            let marker = &text[i..i + ticks];
            if let Some(rel_end) = text[i + ticks..].find(marker) {
                flush_plain(job, plain_start, i);
                let code = text[i + ticks..i + ticks + rel_end].trim();
                job.append(&format!(" {code} "), 0.0, code_fmt.clone());
                i = i + ticks + rel_end + ticks;
                plain_start = i;
                continue;
            }
        }

        // Autolink <https://...> or <mailto:...>
        if bytes[i] == b'<'
            && (text[i + 1..].starts_with("https://")
                || text[i + 1..].starts_with("http://")
                || text[i + 1..].starts_with("mailto:"))
        {
            if let Some(end_rel) = text[i + 1..].find('>') {
                let url = &text[i + 1..i + 1 + end_rel];
                if !url.chars().any(char::is_whitespace) {
                    flush_plain(job, plain_start, i);
                    let mut ls = style;
                    ls.link = true;
                    let shown = url.strip_prefix("mailto:").unwrap_or(url);
                    job.append(shown, 0.0, ls.to_text_format());
                    links.push((shown.to_string(), url.to_string()));
                    i = i + 2 + end_rel;
                    plain_start = i;
                    continue;
                }
            }
        }

        // Bold + Italic ***...*** or ___...___
        if text[i..].starts_with("***") {
            if let Some(rel_end) = text[i + 3..].find("***") {
                if rel_end > 0 {
                    flush_plain(job, plain_start, i);
                    let mut next_style = style;
                    next_style.bold = true;
                    next_style.italic = true;
                    append_inline_spans(&text[i + 3..i + 3 + rel_end], next_style, depth + 1, job, links);
                    i = i + 6 + rel_end;
                    plain_start = i;
                    continue;
                }
            }
        }
        if text[i..].starts_with("___") && is_word_boundary_before(text, i) {
            if let Some(rel_end) = text[i + 3..].find("___") {
                let after_idx = i + 6 + rel_end;
                if rel_end > 0 && is_word_boundary_after(text, after_idx) {
                    flush_plain(job, plain_start, i);
                    let mut next_style = style;
                    next_style.bold = true;
                    next_style.italic = true;
                    append_inline_spans(&text[i + 3..i + 3 + rel_end], next_style, depth + 1, job, links);
                    i = after_idx;
                    plain_start = i;
                    continue;
                }
            }
        }

        // Bold **...** or __...__
        if text[i..].starts_with("**") {
            if let Some(rel_end) = text[i + 2..].find("**") {
                if rel_end > 0 {
                    flush_plain(job, plain_start, i);
                    let mut next_style = style;
                    next_style.bold = true;
                    append_inline_spans(&text[i + 2..i + 2 + rel_end], next_style, depth + 1, job, links);
                    i = i + 4 + rel_end;
                    plain_start = i;
                    continue;
                }
            }
        }
        if text[i..].starts_with("__") && is_word_boundary_before(text, i) {
            if let Some(rel_end) = text[i + 2..].find("__") {
                let after_idx = i + 4 + rel_end;
                if rel_end > 0 && is_word_boundary_after(text, after_idx) {
                    flush_plain(job, plain_start, i);
                    let mut next_style = style;
                    next_style.bold = true;
                    append_inline_spans(&text[i + 2..i + 2 + rel_end], next_style, depth + 1, job, links);
                    i = after_idx;
                    plain_start = i;
                    continue;
                }
            }
        }

        // Strikethrough ~~...~~
        if text[i..].starts_with("~~") {
            if let Some(rel_end) = text[i + 2..].find("~~") {
                if rel_end > 0 {
                    flush_plain(job, plain_start, i);
                    let mut next_style = style;
                    next_style.strike = true;
                    append_inline_spans(&text[i + 2..i + 2 + rel_end], next_style, depth + 1, job, links);
                    i = i + 4 + rel_end;
                    plain_start = i;
                    continue;
                }
            }
        }

        // Nested badge link [![alt](img_url)](link_url)
        if !style.link && text[i..].starts_with("[![") {
            if let Some((alt, link_url, consumed)) = parse_nested_badge_link(&text[i..]) {
                flush_plain(job, plain_start, i);
                let shown = if alt.is_empty() { link_url } else { alt };
                let mut ls = style;
                ls.link = true;
                job.append(shown, 0.0, ls.to_text_format());
                links.push((shown.to_string(), link_url.to_string()));
                i += consumed;
                plain_start = i;
                continue;
            }
        }

        // Markdown link [label](url) or image ![alt](url) with balanced parentheses in URL
        let is_img = bytes[i] == b'!' && text[i..].starts_with("![");
        if !style.link && (bytes[i] == b'[' || is_img) {
            if let Some((label, url_opt, consumed)) = parse_md_link_or_img(&text[i..], is_img) {
                flush_plain(job, plain_start, i);
                if let Some(url) = url_opt {
                    let mut ls = style;
                    ls.link = true;
                    let shown = if label.is_empty() { url } else { label };
                    append_inline_spans(shown, ls, depth + 1, job, links);
                    links.push((strip_inline_markers(shown), url.to_string()));
                } else if !label.is_empty() {
                    // Relative or anchor link like [Overview](#overview) -> render readable label
                    append_inline_spans(label, style, depth + 1, job, links);
                }
                i += consumed;
                plain_start = i;
                continue;
            }
        }

        // Bare URL https://... or http://...
        if !style.link
            && (text[i..].starts_with("https://") || text[i..].starts_with("http://"))
            && is_word_boundary_before(text, i)
        {
            let raw_url = take_bare_url(&text[i..]);
            if raw_url.len() > 8 {
                flush_plain(job, plain_start, i);
                let mut ls = style;
                ls.link = true;
                job.append(raw_url, 0.0, ls.to_text_format());
                links.push((raw_url.to_string(), raw_url.to_string()));
                i += raw_url.len();
                plain_start = i;
                continue;
            }
        }

        // Italic *...* (single asterisk, not followed by space)
        if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] != b' ' && bytes[i + 1] != b'*'
        {
            if let Some(rel_end) = text[i + 1..].find('*') {
                if rel_end > 0 {
                    flush_plain(job, plain_start, i);
                    let mut next_style = style;
                    next_style.italic = true;
                    append_inline_spans(&text[i + 1..i + 1 + rel_end], next_style, depth + 1, job, links);
                    i = i + 2 + rel_end;
                    plain_start = i;
                    continue;
                }
            }
        }

        // Italic _..._ (word-boundary guarded so snake_case_identifiers stay intact)
        if bytes[i] == b'_'
            && i + 1 < bytes.len()
            && bytes[i + 1] != b' '
            && bytes[i + 1] != b'_'
            && is_word_boundary_before(text, i)
        {
            if let Some(rel_end) = text[i + 1..].find('_') {
                let after_idx = i + 2 + rel_end;
                if rel_end > 0 && is_word_boundary_after(text, after_idx) {
                    flush_plain(job, plain_start, i);
                    let mut next_style = style;
                    next_style.italic = true;
                    append_inline_spans(&text[i + 1..i + 1 + rel_end], next_style, depth + 1, job, links);
                    i = after_idx;
                    plain_start = i;
                    continue;
                }
            }
        }

        let ch_len = text[i..].chars().next().map_or(1, char::len_utf8);
        i += ch_len;
    }
    flush_plain(job, plain_start, bytes.len());
}

fn strip_inline_markers(s: &str) -> String {
    s.replace("***", "")
        .replace("**", "")
        .replace("~~", "")
        .replace('`', "")
        .trim()
        .to_string()
}

fn is_word_boundary_before(text: &str, idx: usize) -> bool {
    if idx == 0 {
        return true;
    }
    text[..idx]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric())
}

fn is_word_boundary_after(text: &str, idx: usize) -> bool {
    if idx >= text.len() {
        return true;
    }
    text[idx..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_alphanumeric())
}

fn take_bare_url(slice: &str) -> &str {
    let end = slice
        .find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\''))
        .unwrap_or(slice.len());
    let mut url = &slice[..end];
    while let Some(last) = url.chars().next_back() {
        if matches!(last, '.' | ',' | ';' | ':' | '!' | '?' | ')') {
            if last == ')' && url.matches('(').count() >= url.matches(')').count() {
                break;
            }
            url = &url[..url.len() - last.len_utf8()];
        } else {
            break;
        }
    }
    url
}

fn find_balanced_paren_end(slice: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in slice.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Some(idx);
                }
                depth -= 1;
            }
            '\n' => return None,
            _ => {}
        }
    }
    None
}

fn parse_nested_badge_link(slice: &str) -> Option<(&str, &str, usize)> {
    let rest = slice.strip_prefix("[![")?;
    let alt_end = rest.find("](")?;
    let alt = &rest[..alt_end];
    let after_alt = &rest[alt_end + 2..];
    let img_url_end = find_balanced_paren_end(after_alt)?;
    let after_img = after_alt[img_url_end + 1..].strip_prefix("](")?;
    let link_end = find_balanced_paren_end(after_img)?;
    let link_url = after_img[..link_end]
        .split_whitespace()
        .next()?
        .trim_matches(|c| c == '<' || c == '>');
    if !is_openable_url(link_url) {
        return None;
    }
    let consumed = 3 + alt_end + 2 + img_url_end + 1 + 2 + link_end + 1;
    Some((alt, link_url, consumed))
}

/// Parses `[label](target)` or `![alt](target)`.
/// Returns `Some((label, Some(url), consumed))` for openable http/https/mailto links,
/// or `Some((label, None, consumed))` for local anchors/relative paths so the label still renders cleanly.
fn parse_md_link_or_img(slice: &str, is_img: bool) -> Option<(&str, Option<&str>, usize)> {
    let prefix_len = if is_img { 2 } else { 1 };
    let rest = &slice[prefix_len..];
    let close_br = rest.find("](")?;
    let label = &rest[..close_br];
    if label.contains('\n') {
        return None;
    }
    let after_br = &rest[close_br + 2..];
    let url_end = find_balanced_paren_end(after_br)?;
    let url_part = after_br[..url_end].trim();
    if url_part.is_empty() {
        return None;
    }
    // Strip optional Markdown link title: (https://... "title")
    let target = url_part
        .split_whitespace()
        .next()?
        .trim_matches(|c| c == '<' || c == '>');
    let consumed = prefix_len + close_br + 2 + url_end + 1;
    if is_openable_url(target) {
        Some((label, Some(target), consumed))
    } else if target.starts_with('#')
        || target.starts_with('/')
        || target.starts_with("./")
        || target.starts_with("../")
    {
        Some((label, None, consumed))
    } else {
        None
    }
}

// --- HTML Support (UTF-8 byte-index safe via to_ascii_lowercase) ---

/// Strips `<!-- ... -->`, `<script>...</script>`, and `<style>...</style>` outside
/// fenced code blocks and inline backtick spans.
fn strip_comments_and_nonvisual(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut buf = String::new();
    let mut in_fence = false;
    let mut fence_ch = b'`';
    let mut fence_len = 3;

    for line in input.split_inclusive('\n') {
        let t = line.trim();
        if !in_fence {
            if let Some((ch, len, _)) = parse_fence_open(t) {
                if !buf.is_empty() {
                    out.push_str(&strip_nonvisual_fragment(&buf));
                    buf.clear();
                }
                in_fence = true;
                fence_ch = ch;
                fence_len = len;
                out.push_str(line);
                continue;
            }
            buf.push_str(line);
        } else {
            out.push_str(line);
            if is_fence_close(t, fence_ch, fence_len) {
                in_fence = false;
            }
        }
    }
    if !buf.is_empty() {
        out.push_str(&strip_nonvisual_fragment(&buf));
    }
    out
}

fn strip_nonvisual_fragment(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let lower = input.to_ascii_lowercase();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Preserve inline code spans verbatim
        if bytes[i] == b'`' {
            let ticks = input[i..].bytes().take_while(|&b| b == b'`').count();
            let marker = &input[i..i + ticks];
            if let Some(rel_end) = input[i + ticks..].find(marker) {
                let end = i + ticks + rel_end + ticks;
                out.push_str(&input[i..end]);
                i = end;
                continue;
            }
        }
        if input[i..].starts_with("<!--") {
            if let Some(end_rel) = input[i + 4..].find("-->") {
                i = i + 4 + end_rel + 3;
                continue;
            } else {
                break;
            }
        }
        if has_tag_open_at(&lower, i, "script") {
            if let Some(end_rel) = lower[i..].find("</script>") {
                i = i + end_rel + "</script>".len();
                continue;
            } else {
                break;
            }
        }
        if has_tag_open_at(&lower, i, "style") {
            if let Some(end_rel) = lower[i..].find("</style>") {
                i = i + end_rel + "</style>".len();
                continue;
            } else {
                break;
            }
        }
        let ch_len = input[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&input[i..i + ch_len]);
        i += ch_len;
    }
    out
}

fn has_tag_open_at(lower: &str, idx: usize, tag: &str) -> bool {
    let rest = &lower[idx..];
    if !rest.starts_with('<') {
        return false;
    }
    let after_lt = &rest[1..];
    if !after_lt.starts_with(tag) {
        return false;
    }
    match after_lt.as_bytes().get(tag.len()) {
        Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r') => true,
        _ => false,
    }
}

fn looks_like_html(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    for needle in [
        "<p>", "<p ", "</p>", "<br", "<hr", "<div", "<span", "<h1", "<h2", "<h3", "<h4", "<h5",
        "<h6", "<ul", "<ol", "<li", "<table", "<tr", "<td", "<th", "<strong", "<b>", "<b ",
        "<em>", "<i>", "<i ", "<code", "<pre", "<kbd", "<a ", "<img ", "<input ", "<details",
        "<summary", "<blockquote", "<del>", "<s>", "<center", "<picture", "<sup>", "<sub>",
    ] {
        if lower.contains(needle) {
            return true;
        }
    }
    false
}

/// Splits top-level `<details>...</details>` blocks with proper nesting depth tracking,
/// ignoring tags inside fenced code blocks or inline backtick spans.
fn split_details_segments(input: &str) -> Vec<BodySegment> {
    let mut out = Vec::new();
    let lower = input.to_ascii_lowercase();
    let mut pos = 0;

    while let Some(rel_start) = find_tag_outside_code(&lower[pos..], "details") {
        let start = pos + rel_start;
        if start > pos {
            out.push(BodySegment::Markup(input[pos..start].to_string()));
        }
        let Some(open_end_rel) = input[start..].find('>') else {
            out.push(BodySegment::Markup(input[start..].to_string()));
            return out;
        };
        let open_tag = &lower[start..start + open_end_rel + 1];
        let is_open = open_tag.contains(" open");
        let content_start = start + open_end_rel + 1;

        let Some(content_end) = find_matching_details_close(&lower, content_start) else {
            out.push(BodySegment::Markup(input[start..].to_string()));
            return out;
        };

        let inner = &input[content_start..content_end];
        let inner_lower = &lower[content_start..content_end];

        // Only match <summary> belonging to THIS <details> (must precede any nested <details>)
        let first_nested = find_tag_outside_code(inner_lower, "details").unwrap_or(inner.len());
        let (summary, body) = if let Some(sum_start) =
            find_tag_outside_code(&inner_lower[..first_nested], "summary")
        {
            if let Some(sum_open_end) = inner[sum_start..].find('>') {
                let s_body_start = sum_start + sum_open_end + 1;
                if let Some(sum_close) = inner_lower[s_body_start..].find("</summary>") {
                    let s_text = inner[s_body_start..s_body_start + sum_close].to_string();
                    let rest = format!(
                        "{}{}",
                        &inner[..sum_start],
                        &inner[s_body_start + sum_close + "</summary>".len()..]
                    );
                    (s_text, rest)
                } else {
                    (String::new(), inner.to_string())
                }
            } else {
                (String::new(), inner.to_string())
            }
        } else {
            (String::new(), inner.to_string())
        };

        out.push(BodySegment::Details {
            summary,
            body,
            open: is_open,
        });
        pos = content_end + "</details>".len();
    }

    if pos < input.len() {
        out.push(BodySegment::Markup(input[pos..].to_string()));
    }
    out
}

fn find_matching_details_close(lower: &str, mut pos: usize) -> Option<usize> {
    let mut depth = 1usize;
    let bytes = lower.as_bytes();
    while pos < bytes.len() {
        // Skip inline code spans
        if bytes[pos] == b'`' {
            let ticks = lower[pos..].bytes().take_while(|&b| b == b'`').count();
            let marker = &lower[pos..pos + ticks];
            if let Some(rel_end) = lower[pos + ticks..].find(marker) {
                pos = pos + ticks + rel_end + ticks;
                continue;
            }
        }
        if has_tag_open_at(lower, pos, "details") {
            depth += 1;
            pos += "<details".len();
            continue;
        }
        if lower[pos..].starts_with("</details>") {
            depth -= 1;
            if depth == 0 {
                return Some(pos);
            }
            pos += "</details>".len();
            continue;
        }
        let ch_len = lower[pos..].chars().next().map_or(1, char::len_utf8);
        pos += ch_len;
    }
    None
}

/// Finds the first `<tag...>` occurrence outside fenced code blocks AND inline backtick spans.
fn find_tag_outside_code(lower: &str, tag: &str) -> Option<usize> {
    let mut in_fence = false;
    let mut fence_ch = b'`';
    let mut fence_len = 3;
    let mut offset = 0;

    for line in lower.split_inclusive('\n') {
        let t = line.trim();
        if !in_fence {
            if let Some((ch, len, _)) = parse_fence_open(t) {
                in_fence = true;
                fence_ch = ch;
                fence_len = len;
            } else {
                let bytes = line.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'`' {
                        let ticks = line[i..].bytes().take_while(|&b| b == b'`').count();
                        let marker = &line[i..i + ticks];
                        if let Some(rel_end) = line[i + ticks..].find(marker) {
                            i = i + ticks + rel_end + ticks;
                            continue;
                        }
                    }
                    if has_tag_open_at(line, i, tag) {
                        return Some(offset + i);
                    }
                    let ch_len = line[i..].chars().next().map_or(1, char::len_utf8);
                    i += ch_len;
                }
            }
        } else if is_fence_close(t, fence_ch, fence_len) {
            in_fence = false;
        }
        offset += line.len();
    }
    None
}

fn html_to_markdown_preserving_fences(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut buf = String::new();
    let mut in_fence = false;
    let mut fence_ch = b'`';
    let mut fence_len = 3;

    for line in input.split_inclusive('\n') {
        let t = line.trim();
        if !in_fence {
            if let Some((ch, len, _)) = parse_fence_open(t) {
                if !buf.is_empty() {
                    out.push_str(&convert_html_fragment(&buf));
                    buf.clear();
                }
                in_fence = true;
                fence_ch = ch;
                fence_len = len;
                out.push_str(line);
                continue;
            }
            buf.push_str(line);
        } else {
            out.push_str(line);
            if is_fence_close(t, fence_ch, fence_len) {
                in_fence = false;
            }
        }
    }
    if !buf.is_empty() {
        out.push_str(&convert_html_fragment(&buf));
    }
    out
}

fn convert_html_fragment(html: &str) -> String {
    let step1 = convert_pre_blocks(html);
    let step2 = convert_html_tables(&step1);
    let step3 = convert_html_blockquotes(&step2);

    let mut out = String::with_capacity(step3.len());
    let bytes = step3.as_bytes();
    let mut i = 0;
    let mut list_stack: Vec<Option<usize>> = Vec::new(); // None = ul, Some(n) = ol counter

    while i < bytes.len() {
        // Preserve fenced code blocks emitted by convert_pre_blocks (or already present)
        if (i == 0 || bytes[i - 1] == b'\n') && step3[i..].starts_with("```") {
            let line_end = step3[i..].find('\n').map_or(step3.len(), |p| i + p + 1);
            let first_line = step3[i..line_end].trim();
            if let Some((fence_ch, fence_len, _)) = parse_fence_open(first_line) {
                out.push_str(&step3[i..line_end]);
                i = line_end;
                while i < bytes.len() {
                    let next_end = step3[i..].find('\n').map_or(step3.len(), |p| i + p + 1);
                    let cur = step3[i..next_end].trim();
                    out.push_str(&step3[i..next_end]);
                    i = next_end;
                    if is_fence_close(cur, fence_ch, fence_len) {
                        break;
                    }
                }
                continue;
            }
        }

        // Preserve inline Markdown backtick spans verbatim (so `<div>` and `&amp;` aren't touched!)
        if bytes[i] == b'`' {
            let ticks = step3[i..].bytes().take_while(|&b| b == b'`').count();
            let marker = &step3[i..i + ticks];
            if let Some(rel_end) = step3[i + ticks..].find(marker) {
                let end = i + ticks + rel_end + ticks;
                out.push_str(&step3[i..end]);
                i = end;
                continue;
            }
        }

        if bytes[i] == b'<' {
            // Leave Markdown autolinks <https://...> / <mailto:...> untouched
            if step3[i + 1..].starts_with("https://")
                || step3[i + 1..].starts_with("http://")
                || step3[i + 1..].starts_with("mailto:")
            {
                out.push('<');
                i += 1;
                continue;
            }
            if let Some(close_rel) = step3[i + 1..].find('>') {
                let raw_tag = &step3[i + 1..i + 1 + close_rel];
                let tag_trim = raw_tag.trim();
                let is_closing = tag_trim.starts_with('/');
                let name_body = tag_trim.trim_start_matches('/').trim_end_matches('/').trim();
                let tag_name = name_body
                    .split(|c: char| c.is_whitespace() || c == '/')
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();

                // ONLY process real HTML tags; leave Rust/TS generics like Vec<String>, Option<T> intact!
                if is_known_html_tag(&tag_name) {
                    let end_idx = i + 1 + close_rel + 1;
                    match (is_closing, tag_name.as_str()) {
                        (false, "a") => {
                            let href = extract_html_attr(tag_trim, "href").unwrap_or_default();
                            let title = extract_html_attr(tag_trim, "title").unwrap_or_default();
                            if let Some(a_close) = find_closing_tag(&step3[end_idx..], "a") {
                                let inner_html = &step3[end_idx..end_idx + a_close];
                                let mut label =
                                    decode_html_entities(&strip_known_html_tags(inner_html))
                                        .replace('\n', " ")
                                        .trim()
                                        .to_string();
                                // Support <a href="..."><img src="..." alt="Badge"></a>
                                if label.is_empty() {
                                    if let Some(alt) = extract_inner_img_alt(inner_html) {
                                        label = alt;
                                    } else if !title.trim().is_empty() {
                                        label = title.trim().to_string();
                                    }
                                }
                                if is_openable_url(&href) {
                                    let shown = if label.is_empty() { &href } else { &label };
                                    out.push_str(&format!("[{shown}]({href})"));
                                } else {
                                    out.push_str(&label);
                                }
                                i = end_idx + a_close + "</a>".len();
                                continue;
                            }
                        }
                        (false, "img") => {
                            let src = extract_html_attr(tag_trim, "src").unwrap_or_default();
                            let alt = extract_html_attr(tag_trim, "alt").unwrap_or_default();
                            if is_openable_url(&src) {
                                let lbl = if alt.trim().is_empty() {
                                    "Image".to_string()
                                } else {
                                    format!("Image: {}", alt.trim())
                                };
                                out.push_str(&format!("[{lbl}]({src})"));
                            } else if !alt.trim().is_empty() {
                                out.push_str(alt.trim());
                            }
                            i = end_idx;
                            continue;
                        }
                        (false, "input") => {
                            let typ = extract_html_attr(tag_trim, "type")
                                .unwrap_or_default()
                                .to_ascii_lowercase();
                            if typ == "checkbox" {
                                let lower_tag = tag_trim.to_ascii_lowercase();
                                let checked = lower_tag.contains(" checked");
                                if checked {
                                    out.push_str("[x] ");
                                } else {
                                    out.push_str("[ ] ");
                                }
                                i = skip_ascii_ws(&step3, end_idx);
                                continue;
                            }
                            i = end_idx;
                            continue;
                        }
                        (false, "b" | "strong" | "i" | "em" | "s" | "del" | "strike" | "code" | "kbd" | "tt" | "samp") => {
                            let wrapper = match tag_name.as_str() {
                                "b" | "strong" => "**",
                                "i" | "em" => "*",
                                "s" | "del" | "strike" => "~~",
                                _ => "`",
                            };
                            if let Some(c_rel) = find_closing_tag(&step3[end_idx..], &tag_name) {
                                let inner_raw = &step3[end_idx..end_idx + c_rel];
                                let inner_text = if wrapper == "`" {
                                    decode_html_entities(&strip_known_html_tags(inner_raw))
                                        .replace('\n', " ")
                                        .trim()
                                        .to_string()
                                } else {
                                    convert_html_fragment(inner_raw)
                                        .replace('\n', " ")
                                        .trim()
                                        .to_string()
                                };
                                if !inner_text.is_empty() {
                                    out.push_str(wrapper);
                                    out.push_str(&inner_text);
                                    out.push_str(wrapper);
                                }
                                i = end_idx + c_rel + tag_name.len() + 3; // </tag>
                                continue;
                            }
                        }
                        (false, "br") => out.push('\n'),
                        (false, "hr") => out.push_str("\n---\n"),
                        (false, "h1") => {
                            out.push_str("\n# ");
                            i = skip_ascii_ws(&step3, end_idx);
                            continue;
                        }
                        (false, "h2") => {
                            out.push_str("\n## ");
                            i = skip_ascii_ws(&step3, end_idx);
                            continue;
                        }
                        (false, "h3") => {
                            out.push_str("\n### ");
                            i = skip_ascii_ws(&step3, end_idx);
                            continue;
                        }
                        (false, "h4" | "h5" | "h6") => {
                            out.push_str("\n#### ");
                            i = skip_ascii_ws(&step3, end_idx);
                            continue;
                        }
                        (true, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") => out.push('\n'),
                        (false, "p" | "div" | "section" | "article" | "header" | "footer") => {
                            out.push('\n')
                        }
                        (true, "p" | "div" | "section" | "article" | "header" | "footer") => {
                            out.push('\n')
                        }
                        (false, "ul") => list_stack.push(None),
                        (false, "ol") => list_stack.push(Some(0)),
                        (true, "ul" | "ol") => {
                            list_stack.pop();
                            out.push('\n');
                        }
                        (false, "li") => {
                            let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                            match list_stack.last_mut() {
                                Some(Some(counter)) => {
                                    *counter += 1;
                                    out.push_str(&format!("\n{indent}{counter}. "));
                                }
                                _ => out.push_str(&format!("\n{indent}- ")),
                            }
                            // Skip leading newlines/spaces inside <li>\n  Item\n</li>
                            i = skip_ascii_ws(&step3, end_idx);
                            continue;
                        }
                        (true, "li") => out.push('\n'),
                        _ => {}
                    }
                    i = end_idx;
                    continue;
                }
            }
        }

        // Decode HTML entity in non-code text immediately (never touches code blocks or backticks)
        if bytes[i] == b'&' {
            if let Some((decoded, consumed)) = try_decode_entity_at(&step3[i..]) {
                out.push_str(&decoded);
                i += consumed;
                continue;
            }
        }

        let ch_len = step3[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&step3[i..i + ch_len]);
        i += ch_len;
    }

    out
}

fn skip_ascii_ws(s: &str, mut idx: usize) -> usize {
    let bytes = s.as_bytes();
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    idx
}

fn extract_inner_img_alt(inner_html: &str) -> Option<String> {
    let lower = inner_html.to_ascii_lowercase();
    let start = lower.find("<img")?;
    let end = inner_html[start..].find('>')?;
    let tag = &inner_html[start + 1..start + end];
    let alt = extract_html_attr(tag, "alt")?;
    if alt.trim().is_empty() {
        None
    } else {
        Some(alt.trim().to_string())
    }
}

fn convert_pre_blocks(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let lower = html.to_ascii_lowercase();
    let mut pos = 0;

    while let Some(rel) = find_tag_outside_code(&lower[pos..], "pre") {
        let start = pos + rel;
        out.push_str(&html[pos..start]);
        let Some(open_end) = html[start..].find('>') else {
            out.push_str(&html[start..]);
            return out;
        };
        let body_start = start + open_end + 1;
        let Some(close_rel) = lower[body_start..].find("</pre>") else {
            out.push_str(&html[start..]);
            return out;
        };
        let inner = &html[body_start..body_start + close_rel];
        let lang = extract_code_lang(inner).unwrap_or_default();
        let code = decode_html_entities(&strip_known_html_tags(inner));
        out.push_str("\n```");
        out.push_str(&lang);
        out.push('\n');
        out.push_str(code.trim_matches('\n'));
        out.push_str("\n```\n");
        pos = body_start + close_rel + "</pre>".len();
    }
    out.push_str(&html[pos..]);
    out
}

fn convert_html_blockquotes(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let lower = html.to_ascii_lowercase();
    let mut pos = 0;

    while let Some(rel) = find_tag_outside_code(&lower[pos..], "blockquote") {
        let start = pos + rel;
        out.push_str(&html[pos..start]);
        let Some(open_end) = html[start..].find('>') else {
            out.push_str(&html[start..]);
            return out;
        };
        let body_start = start + open_end + 1;
        let Some(close_rel) = lower[body_start..].find("</blockquote>") else {
            out.push_str(&html[start..]);
            return out;
        };
        let inner = &html[body_start..body_start + close_rel];
        let inner_md = convert_html_fragment(inner);
        out.push('\n');
        for line in inner_md.trim().lines() {
            out.push_str("> ");
            out.push_str(line.trim());
            out.push('\n');
        }
        out.push('\n');
        pos = body_start + close_rel + "</blockquote>".len();
    }
    out.push_str(&html[pos..]);
    out
}

fn extract_code_lang(inner_pre: &str) -> Option<String> {
    let lower = inner_pre.to_ascii_lowercase();
    let start = lower.find("<code")?;
    let end = inner_pre[start..].find('>')?;
    let cls = extract_html_attr(&inner_pre[start + 1..start + end], "class")?;
    for tok in cls.split_whitespace() {
        if let Some(lang) = tok
            .strip_prefix("language-")
            .or_else(|| tok.strip_prefix("lang-"))
        {
            return Some(lang.to_string());
        }
    }
    None
}

fn convert_html_tables(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let lower = html.to_ascii_lowercase();
    let mut pos = 0;

    while let Some(rel) = find_tag_outside_code(&lower[pos..], "table") {
        let start = pos + rel;
        out.push_str(&html[pos..start]);
        let Some(open_end) = html[start..].find('>') else {
            out.push_str(&html[start..]);
            return out;
        };
        let body_start = start + open_end + 1;
        let Some(close_rel) = lower[body_start..].find("</table>") else {
            out.push_str(&html[start..]);
            return out;
        };
        let tbl = &html[body_start..body_start + close_rel];
        let tbl_lower = &lower[body_start..body_start + close_rel];

        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut rpos = 0;
        while let Some(tr_rel) = tbl_lower[rpos..].find("<tr") {
            let tr_start = rpos + tr_rel;
            let Some(tr_open_end) = tbl[tr_start..].find('>') else {
                break;
            };
            let tr_body_start = tr_start + tr_open_end + 1;
            let Some(tr_close_rel) = tbl_lower[tr_body_start..].find("</tr>") else {
                break;
            };
            let row_slice = &tbl[tr_body_start..tr_body_start + tr_close_rel];
            let row_lower = &tbl_lower[tr_body_start..tr_body_start + tr_close_rel];
            let mut cells = Vec::new();
            let mut cpos = 0;
            while cpos < row_slice.len() {
                let next_td = row_lower[cpos..].find("<td");
                let next_th = row_lower[cpos..].find("<th");
                let next = match (next_td, next_th) {
                    (Some(a), Some(b)) => Some((a.min(b), a <= b)),
                    (Some(a), None) => Some((a, true)),
                    (None, Some(b)) => Some((b, false)),
                    (None, None) => None,
                };
                let Some((c_rel, is_td)) = next else {
                    break;
                };
                let c_start = cpos + c_rel;
                let Some(c_open_end) = row_slice[c_start..].find('>') else {
                    break;
                };
                let c_body_start = c_start + c_open_end + 1;
                let close_tag = if is_td { "</td>" } else { "</th>" };
                let Some(c_close_rel) = row_lower[c_body_start..].find(close_tag) else {
                    break;
                };
                let cell_raw = &row_slice[c_body_start..c_body_start + c_close_rel];
                let cell_clean = convert_html_fragment(cell_raw)
                    .replace('\n', " ")
                    .replace('|', "\\|")
                    .trim()
                    .to_string();
                cells.push(cell_clean);
                cpos = c_body_start + c_close_rel + close_tag.len();
            }
            if !cells.is_empty() {
                rows.push(cells);
            }
            rpos = tr_body_start + tr_close_rel + "</tr>".len();
        }

        if !rows.is_empty() {
            out.push('\n');
            for (idx, row) in rows.iter().enumerate() {
                out.push_str("| ");
                out.push_str(&row.join(" | "));
                out.push_str(" |\n");
                if idx == 0 {
                    out.push('|');
                    for _ in row {
                        out.push_str(" --- |");
                    }
                    out.push('\n');
                }
            }
            out.push('\n');
        }
        pos = body_start + close_rel + "</table>".len();
    }
    out.push_str(&html[pos..]);
    out
}

fn find_closing_tag(slice: &str, tag: &str) -> Option<usize> {
    let needle = format!("</{tag}>");
    slice.to_ascii_lowercase().find(&needle)
}

/// Extracts an HTML attribute while ensuring exact attribute-name boundaries
/// (so `data-src="..."` never matches `src`, and `data-alt="..."` never matches `alt`).
fn extract_html_attr(tag_contents: &str, attr_name: &str) -> Option<String> {
    let lower = tag_contents.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut search_from = 0;

    while let Some(rel) = lower[search_from..].find(attr_name) {
        let start = search_from + rel;
        let boundary_ok = start == 0 || bytes[start - 1].is_ascii_whitespace();
        let after_name = start + attr_name.len();
        if boundary_ok {
            let mut eq_idx = after_name;
            while eq_idx < bytes.len() && bytes[eq_idx].is_ascii_whitespace() {
                eq_idx += 1;
            }
            if eq_idx < bytes.len() && bytes[eq_idx] == b'=' {
                let rest = tag_contents[eq_idx + 1..].trim_start();
                if let Some(q) = rest.chars().next().filter(|&c| c == '"' || c == '\'') {
                    let inner = &rest[1..];
                    let end = inner.find(q)?;
                    return Some(decode_html_entities(&inner[..end]));
                } else {
                    let end = rest
                        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                        .unwrap_or(rest.len());
                    return Some(decode_html_entities(&rest[..end]));
                }
            }
        }
        search_from = after_name;
    }
    None
}

fn strip_known_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some(close_rel) = input[i + 1..].find('>') {
                let raw_tag = input[i + 1..i + 1 + close_rel].trim();
                let name = raw_tag
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .split(|c: char| c.is_whitespace() || c == '/')
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if is_known_html_tag(&name) {
                    i += close_rel + 2;
                    continue;
                }
            }
        }
        let ch_len = input[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&input[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Decodes HTML entities outside fenced code blocks and inline backtick spans (for pure Markdown).
fn decode_entities_outside_code(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_fence = false;
    let mut fence_ch = b'`';
    let mut fence_len = 3;

    for line in input.split_inclusive('\n') {
        let t = line.trim();
        if !in_fence {
            if let Some((ch, len, _)) = parse_fence_open(t) {
                in_fence = true;
                fence_ch = ch;
                fence_len = len;
                out.push_str(line);
                continue;
            }
            let bytes = line.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'`' {
                    let ticks = line[i..].bytes().take_while(|&b| b == b'`').count();
                    let marker = &line[i..i + ticks];
                    if let Some(rel_end) = line[i + ticks..].find(marker) {
                        let end = i + ticks + rel_end + ticks;
                        out.push_str(&line[i..end]);
                        i = end;
                        continue;
                    }
                }
                if bytes[i] == b'&' {
                    if let Some((decoded, consumed)) = try_decode_entity_at(&line[i..]) {
                        out.push_str(&decoded);
                        i += consumed;
                        continue;
                    }
                }
                let ch_len = line[i..].chars().next().map_or(1, char::len_utf8);
                out.push_str(&line[i..i + ch_len]);
                i += ch_len;
            }
        } else {
            out.push_str(line);
            if is_fence_close(t, fence_ch, fence_len) {
                in_fence = false;
            }
        }
    }
    out
}

fn decode_html_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some((decoded, consumed)) = try_decode_entity_at(&input[i..]) {
                out.push_str(&decoded);
                i += consumed;
                continue;
            }
        }
        let ch_len = input[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&input[i..i + ch_len]);
        i += ch_len;
    }
    out
}

fn try_decode_entity_at(slice: &str) -> Option<(String, usize)> {
    let rest = slice.strip_prefix('&')?;
    let semi_rel = rest.find(';')?;
    if semi_rel > 12 {
        return None;
    }
    let decoded = decode_single_entity(&rest[..semi_rel])?;
    Some((decoded, semi_rel + 2))
}

fn decode_single_entity(entity: &str) -> Option<String> {
    if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        let code = u32::from_str_radix(hex, 16).ok()?;
        return char::from_u32(code).map(|c| c.to_string());
    }
    if let Some(dec) = entity.strip_prefix('#') {
        let code = dec.parse::<u32>().ok()?;
        return char::from_u32(code).map(|c| c.to_string());
    }
    let s = match entity {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => " ",
        "mdash" => "—",
        "ndash" => "–",
        "hellip" => "...",
        "rarr" => "→",
        "larr" => "←",
        "copy" => "©",
        "times" => "×",
        _ => return None,
    };
    Some(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_html_tags_list_is_strictly_sorted() {
        for pair in KNOWN_HTML_TAGS.windows(2) {
            assert!(
                pair[0] < pair[1],
                "KNOWN_HTML_TAGS out of order: {} >= {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn handles_unicode_uppercase_without_byte_offset_panic() {
        // Turkish capital I with dot (İ, U+0130) expands from 2 bytes to 3 bytes under Unicode to_lowercase()
        let tricky = "İstanbul <details open><summary>İnfo</summary><pre><code class=\"language-rust\">let x = 1;</code></pre><table><tr><th>İtem</th></tr><tr><td>Ok</td></tr></table></details>";
        let segs = split_details_segments(tricky);
        assert_eq!(segs.len(), 2);
        if let BodySegment::Details { summary, body, .. } = &segs[1] {
            assert_eq!(summary, "İnfo");
            let md = html_to_markdown_preserving_fences(body);
            assert!(md.contains("```rust\nlet x = 1;\n```"));
            assert!(md.contains("| İtem |"));
        } else {
            panic!("expected Details");
        }
    }

    #[test]
    fn handles_nested_details_and_summary_scoping() {
        let nested = "Top\n<details><summary>Outer</summary>Before<details><summary>Inner</summary>Deep</details>After</details>\nBottom";
        let segs = split_details_segments(nested);
        assert_eq!(segs.len(), 3);
        if let BodySegment::Details { summary, body, .. } = &segs[1] {
            assert_eq!(summary, "Outer");
            assert!(body.contains("<details><summary>Inner</summary>Deep</details>"));
            assert!(body.contains("After"));
        } else {
            panic!("expected Outer Details");
        }

        // Outer details without its own summary must NOT steal inner summary
        let no_outer_sum = "<details><details><summary>InnerOnly</summary>Body</details></details>";
        let segs2 = split_details_segments(no_outer_sum);
        assert_eq!(segs2.len(), 1);
        if let BodySegment::Details { summary, body, .. } = &segs2[0] {
            assert!(summary.is_empty());
            assert!(body.contains("<summary>InnerOnly</summary>"));
        } else {
            panic!("expected Details");
        }

        // Inline code `<details>` must NOT be split as an HTML details block
        let inline_code = "Use `<details>` and `<summary>` tags in HTML.";
        let segs3 = split_details_segments(inline_code);
        assert_eq!(segs3.len(), 1);
        assert!(matches!(segs3[0], BodySegment::Markup(_)));
    }

    #[test]
    fn preserves_generics_comments_and_entities_inside_code() {
        let text = "<!-- top comment -->\n```html\n<!-- keep comment -->\n<div>&amp;</div>\n```\n<p>Returns Result<User> and `<span>&nbsp;</span>` &mdash; ok</p>";
        let cleaned = strip_comments_and_nonvisual(text);
        assert!(!cleaned.contains("top comment"));
        assert!(cleaned.contains("<!-- keep comment -->"));
        let md = html_to_markdown_preserving_fences(&cleaned);
        assert!(md.contains("Result<User>"));
        assert!(md.contains("`<span>&nbsp;</span>`"));
        assert!(md.contains("— ok"));
        assert!(md.contains("<div>&amp;</div>"));
    }

    #[test]
    fn handles_multiline_html_lists_headings_blockquotes_and_attrs() {
        let html = "<h1>\n  Title\n</h1>\n<ul>\n  <li>\n    First item\n  </li>\n  <li><input type=\"checkbox\" checked> Done task</li>\n</ul>\n<blockquote><p>Line 1</p><p>Line 2</p></blockquote>";
        let md = html_to_markdown_preserving_fences(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("- First item"));
        assert!(md.contains("- [x] Done task"));
        assert!(md.contains("> Line 1"));
        assert!(md.contains("> Line 2"));

        // Exact attribute boundary check: data-src must NOT collide with src
        let img = r#"img data-src="https://wrong.example" src="https://right.example/a.png" data-alt="Wrong" alt="Right""#;
        assert_eq!(
            extract_html_attr(img, "src").as_deref(),
            Some("https://right.example/a.png")
        );
        assert_eq!(extract_html_attr(img, "alt").as_deref(), Some("Right"));
    }

    #[test]
    fn parses_html_ordered_lists_linked_badges_and_entities() {
        let html = "<ol><li>First &#x27;quoted&#x27; &mdash; ok</li><li>Second</li></ol><a href=\"https://docs.example.com\"><img src=\"https://img.example.com/b.svg\" alt=\"Build Passing\"></a>";
        let md = html_to_markdown_preserving_fences(html);
        assert!(md.contains("1. First 'quoted' — ok"));
        assert!(md.contains("2. Second"));
        assert!(md.contains("[Build Passing](https://docs.example.com)"));
    }

    #[test]
    fn parses_dependabot_badges_and_hidden_comments() {
        assert!(is_hidden_md_comment("[//]: # (dependabot-automerge-start)"));
        assert!(is_hidden_md_comment("[comment]: <> (note)"));
        let badge = "[![Dependabot compatibility score](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=foo)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)";
        let parsed = parse_nested_badge_link(badge).expect("nested badge parsed");
        assert_eq!(parsed.0, "Dependabot compatibility score");
        assert!(parsed.1.starts_with("https://docs.github.com/"));
        assert_eq!(parsed.2, badge.len());
    }

    #[test]
    fn resolves_reference_links_balanced_urls_and_local_anchors() {
        let src = "See [Guide][g] and [Overview](#overview).\n\n[g]: https://en.wikipedia.org/wiki/Rust_(programming_language)\n";
        let refs = collect_reference_links(src);
        let resolved = resolve_reference_links(src, &refs);
        assert!(resolved.contains("[Guide](https://en.wikipedia.org/wiki/Rust_(programming_language))"));
        let link = parse_md_link_or_img(
            "[Guide](https://en.wikipedia.org/wiki/Rust_(programming_language))",
            false,
        )
        .expect("balanced paren url");
        assert_eq!(link.0, "Guide");
        assert_eq!(
            link.1,
            Some("https://en.wikipedia.org/wiki/Rust_(programming_language)")
        );
        let anchor = parse_md_link_or_img("[Overview](#overview)", false).expect("anchor link");
        assert_eq!(anchor.0, "Overview");
        assert_eq!(anchor.1, None);
    }

    #[test]
    fn handles_tables_escaped_pipes_headings_and_setext_guards() {
        let cells = split_table_row(r"| Operator | `a | b` | a \| b |");
        assert_eq!(cells, vec!["Operator", "`a | b`", "a | b"]);

        // Single pipe line without separator row must NOT be treated as a table
        assert!(!is_table_start(&["|x| = 5"], 0));
        assert!(is_table_start(&["| A | B |", "| --- | --- |"], 0));

        // Trailing '#' in C# heading must NOT be stripped, whereas 'Title ##' must be
        assert_eq!(strip_optional_closing_hashes("Migration to C#"), "Migration to C#");
        assert_eq!(strip_optional_closing_hashes("Section Title ##"), "Section Title");

        // Bullet followed by --- must NOT turn the bullet into a Setext heading
        assert!(!is_setext_heading_candidate("- List item"));
        assert!(is_setext_heading_candidate("Real Heading"));
    }

    #[test]
    fn strips_script_and_style_blocks_safely() {
        let raw = "<style>.a{color:red}</style><p>Safe</p><script>alert(1)</script>";
        let cleaned = strip_comments_and_nonvisual(raw);
        assert!(!cleaned.contains("color:red"));
        assert!(!cleaned.contains("alert(1)"));
        assert!(cleaned.contains("<p>Safe</p>"));
    }

    #[test]
    fn nested_inline_formatting_extracts_links_and_styles() {
        let mut job = LayoutJob::default();
        let mut links = Vec::new();
        let style = InlineStyle {
            size: 12.0,
            base_color: theme::TEXT_PRIMARY,
            bold: false,
            italic: false,
            strike: false,
            link: false,
        };
        append_inline_spans(
            "Check **[`run()`](https://example.com/run)** and ***bold italic*** with snake_case_var",
            style,
            0,
            &mut job,
            &mut links,
        );
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, "run()");
        assert_eq!(links[0].1, "https://example.com/run");
        assert!(job.text.contains("snake_case_var"));
        assert!(job.text.contains("bold italic"));
        assert!(!job.text.contains("**"));
    }
}
