/// Distribution visualiser — produces ASCII histograms for the CLI and
/// inline SVG bar charts for the web playground.
///
/// The two render functions are pure: all distribution data has already
/// been reduced to (label, probability) pairs by the interpreter.

// ── Data Types ────────────────────────────────────────────────────────────────

/// All the information needed to draw a histogram for one distribution.
#[derive(Debug, Clone)]
pub struct HistogramData {
    /// Human-readable distribution label, e.g. `"uniform(1, 6)"`.
    pub label: String,
    /// Whether the distribution is discrete (PMF bars) or continuous.
    pub kind: HistKind,
    /// `(outcome_label, prob_f64, prob_display)` triples, sorted ascending by outcome.
    /// `prob_f64` is used only for bar-width calculations; `prob_display` is shown to the user.
    /// Empty for continuous distributions.
    pub bars: Vec<(String, f64, String)>,
}

#[derive(Debug, Clone)]
pub enum HistKind {
    Discrete,
    Continuous { min: f64, max: f64, mean: f64 },
}

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum number of bars rendered before truncating.
const MAX_BARS: usize = 40;
/// Width of the bar track in the CLI renderer (in block characters).
const CLI_BAR_W: usize = 30;
/// SVG total width (pixels).
const SVG_W: i32 = 480;
/// X position of the Y-axis line in the SVG.
const SVG_LEFT: i32 = 72;
/// Width of the bar area in the SVG (pixels).
const SVG_BAR_W: i32 = 248;
/// Y coordinate where bars start (below the title/subtitle).
const SVG_TOP: i32 = 50;
/// Height of each row (bar + gap) in the SVG.
const SVG_ROW_H: i32 = 26;
/// Height of each bar rectangle.
const SVG_BAR_H: i32 = 20;
/// Vertical padding above each bar inside its row.
const SVG_BAR_PAD: i32 = 3;
/// Accent colour used for bars.
const ACCENT: &str = "#6366f1";
/// Dimmed version of the accent colour used for the bar track.
const ACCENT_DIM: &str = "rgba(99,102,241,0.13)";

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Escape characters that are special in HTML/SVG text content.
pub fn html_esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Format a float value cleanly: integer if whole, otherwise up to 4 decimal
/// places without trailing zeros.
fn fmt_f(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e14 {
        format!("{}", v as i64)
    } else {
        // up to 4 sig-frac digits, strip trailing zeros
        let s = format!("{:.4}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

// ── CLI Renderer ──────────────────────────────────────────────────────────────

/// Render a distribution as a plain-text ASCII histogram.
///
/// ```
///   uniform(1, 6)  ·  P(X = v)
///
///   1 │██████████████████████████████  0.1667
///   2 │██████████████████████████████  0.1667
///   ...
///     └──────────────────────────────
/// ```
pub fn render_cli(data: &HistogramData) -> String {
    let mut out = String::new();
    out.push('\n');

    match &data.kind {
        HistKind::Continuous { min, max, mean } => {
            out.push_str(&format!("  {}  ·  Continuous Uniform Distribution\n\n", data.label));
            out.push_str(&format!("  min  = {}\n", fmt_f(*min)));
            out.push_str(&format!("  max  = {}\n", fmt_f(*max)));
            out.push_str(&format!("  mean = {}\n", fmt_f(*mean)));
        }

        HistKind::Discrete => {
            out.push_str(&format!("  {}  ·  P(X = v)\n\n", data.label));

            if data.bars.is_empty() {
                out.push_str("  (no outcomes)\n");
                out.push('\n');
                return out;
            }

            let bars: &[(String, f64, String)] = if data.bars.len() > MAX_BARS {
                &data.bars[..MAX_BARS]
            } else {
                &data.bars
            };

            let max_prob = bars.iter().map(|(_, p, _)| *p).fold(0.0_f64, f64::max);
            let max_lbl = bars.iter().map(|(l, _, _)| l.len()).max().unwrap_or(1);

            for (label, prob, display) in bars {
                let filled = if max_prob > 0.0 {
                    ((prob / max_prob) * CLI_BAR_W as f64).round() as usize
                } else {
                    0
                };
                let empty = CLI_BAR_W - filled;
                let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
                out.push_str(&format!(
                    "  {:>w$} │{}  {}\n",
                    label, bar, display,
                    w = max_lbl,
                ));
            }

            if data.bars.len() > MAX_BARS {
                out.push_str(&format!(
                    "  ... ({} more outcomes not shown)\n",
                    data.bars.len() - MAX_BARS
                ));
            }

            let indent = " ".repeat(max_lbl + 4);
            out.push_str(&format!("{}└{}\n", indent, "─".repeat(CLI_BAR_W)));
        }
    }

    out.push('\n');
    out
}

// ── SVG Renderer ──────────────────────────────────────────────────────────────

/// Render a distribution as an inline SVG element suitable for web output.
///
/// `idx` is used to generate unique gradient IDs when multiple visualisations
/// appear in the same HTML document.
///
/// Text uses `currentColor` so it adapts to light/dark theme automatically.
/// Bars use the fixed accent colour `#6366f1`.
pub fn render_svg(data: &HistogramData, idx: usize) -> String {
    match &data.kind {
        HistKind::Continuous { min, max, mean } => render_svg_continuous(data, idx, *min, *max, *mean),
        HistKind::Discrete => render_svg_discrete(data),
    }
}

fn render_svg_continuous(data: &HistogramData, idx: usize, min: f64, max: f64, mean: f64) -> String {
    let h = 140i32;
    let gid = format!("yappl-cg-{}", idx);
    let mid_x = SVG_W / 2;
    let bar_x = SVG_LEFT;
    // Wider bar for continuous (spans more of the viewbox)
    let bar_w = SVG_BAR_W + 60;
    let bar_y = 52i32;
    let bar_h = 22i32;
    // Mean tick x position
    let range = max - min;
    let mean_frac = if range > 0.0 { (mean - min) / range } else { 0.5 };
    let mean_x = bar_x + (mean_frac * bar_w as f64).round() as i32;
    let label_y = bar_y + bar_h + 20;

    let mut s = svg_open(SVG_W, h);
    // Gradient definition
    s.push_str(&format!(
        r#"<defs><linearGradient id="{gid}"><stop offset="0%" stop-color="{ACCENT}" stop-opacity="0.2"/><stop offset="50%" stop-color="{ACCENT}" stop-opacity="0.65"/><stop offset="100%" stop-color="{ACCENT}" stop-opacity="0.2"/></linearGradient></defs>"#
    ));
    // Title
    svg_title(&mut s, mid_x, &data.label, "Continuous Uniform Distribution");
    // Background track
    s.push_str(&format!(
        r#"<rect x="{bar_x}" y="{bar_y}" width="{bar_w}" height="{bar_h}" rx="4" fill="{ACCENT_DIM}"/>"#
    ));
    // Gradient bar (full width for continuous — probability is uniform)
    s.push_str(&format!(
        r#"<rect x="{bar_x}" y="{bar_y}" width="{bar_w}" height="{bar_h}" rx="4" fill="url(#{gid})"/>"#
    ));
    // Mean tick line
    s.push_str(&format!(
        r#"<line x1="{mean_x}" y1="{}" x2="{mean_x}" y2="{}" stroke="{ACCENT}" stroke-width="2.5" stroke-linecap="round"/>"#,
        bar_y - 5,
        bar_y + bar_h + 5
    ));
    // Labels
    s.push_str(&format!(
        r#"<text x="{bar_x}" y="{label_y}" fill="currentColor" font-size="12" opacity="0.75">min = {}</text>"#,
        html_esc(&fmt_f(min))
    ));
    s.push_str(&format!(
        r#"<text x="{mean_x}" y="{label_y}" text-anchor="middle" fill="{ACCENT}" font-size="12" font-weight="600">mean = {}</text>"#,
        html_esc(&fmt_f(mean))
    ));
    let bar_end = bar_x + bar_w;
    s.push_str(&format!(
        r#"<text x="{bar_end}" y="{label_y}" text-anchor="end" fill="currentColor" font-size="12" opacity="0.75">max = {}</text>"#,
        html_esc(&fmt_f(max))
    ));
    s.push_str("</svg>");
    s
}

fn render_svg_discrete(data: &HistogramData) -> String {
    if data.bars.is_empty() {
        // Degenerate: no outcomes
        let h = 80i32;
        let mut s = svg_open(SVG_W, h);
        svg_title(&mut s, SVG_W / 2, &data.label, "Discrete Distribution · P(X = v)");
        s.push_str(&format!(
            r#"<text x="{}" y="65" text-anchor="middle" fill="currentColor" font-size="12" opacity="0.6">(no outcomes)</text>"#,
            SVG_W / 2
        ));
        s.push_str("</svg>");
        return s;
    }

    let bars: &[(String, f64, String)] = if data.bars.len() > MAX_BARS {
        &data.bars[..MAX_BARS]
    } else {
        &data.bars
    };
    let truncated = data.bars.len() > MAX_BARS;
    let n = bars.len() as i32;
    let extra_h = if truncated { 22 } else { 0 };
    let h = SVG_TOP + n * SVG_ROW_H + 8 + 16 + extra_h;
    let max_prob = bars.iter().map(|(_, p, _)| *p).fold(0.0_f64, f64::max);
    let axis_bottom = SVG_TOP + n * SVG_ROW_H + 6;

    let mut s = svg_open(SVG_W, h);
    svg_title(&mut s, SVG_W / 2, &data.label, "Discrete Distribution  ·  P(X = v)");

    // Vertical axis line
    s.push_str(&format!(
        r#"<line x1="{SVG_LEFT}" y1="{SVG_TOP}" x2="{SVG_LEFT}" y2="{axis_bottom}" stroke="currentColor" stroke-width="1" opacity="0.2"/>"#
    ));

    // Bars
    for (i, (label, prob, display)) in bars.iter().enumerate() {
        let y = SVG_TOP + i as i32 * SVG_ROW_H;
        let bw = if max_prob > 0.0 {
            ((prob / max_prob) * SVG_BAR_W as f64).round() as i32
        } else {
            0
        };
        let bar_top = y + SVG_BAR_PAD;
        let text_y = bar_top + SVG_BAR_H - 4;

        // Track (full width, dimmed)
        s.push_str(&format!(
            r#"<rect x="{SVG_LEFT}" y="{bar_top}" width="{SVG_BAR_W}" height="{SVG_BAR_H}" fill="{ACCENT_DIM}"/>"#
        ));
        // Filled bar
        if bw > 0 {
            s.push_str(&format!(
                r#"<rect x="{SVG_LEFT}" y="{bar_top}" width="{bw}" height="{SVG_BAR_H}" fill="{ACCENT}"/>"#
            ));
        }
        // Outcome label (left of axis)
        s.push_str(&format!(
            r#"<text x="{}" y="{text_y}" text-anchor="end" fill="currentColor" font-size="12">{}</text>"#,
            SVG_LEFT - 6,
            html_esc(label)
        ));
        // Probability (right of bar)
        let prob_x = SVG_LEFT + bw + 6;
        s.push_str(&format!(
            r#"<text x="{prob_x}" y="{text_y}" fill="currentColor" font-size="11" opacity="0.72">{}</text>"#,
            html_esc(display)
        ));
    }

    // Horizontal baseline
    s.push_str(&format!(
        r#"<line x1="{SVG_LEFT}" y1="{axis_bottom}" x2="{}" y2="{axis_bottom}" stroke="currentColor" stroke-width="1" opacity="0.2"/>"#,
        SVG_LEFT + SVG_BAR_W
    ));

    // Truncation note
    if truncated {
        s.push_str(&format!(
            r#"<text x="{SVG_LEFT}" y="{}" fill="currentColor" font-size="11" opacity="0.5">… {} more outcomes not shown</text>"#,
            axis_bottom + 16,
            data.bars.len() - MAX_BARS
        ));
    }

    s.push_str("</svg>");
    s
}

// ── Shared SVG helpers ────────────────────────────────────────────────────────

fn svg_open(w: i32, h: i32) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" style="display:block;width:100%;max-width:{w}px;margin:6px 0 12px;font-family:Consolas,'Fira Code',monospace">"#
    )
}

fn svg_title(s: &mut String, mid_x: i32, label: &str, subtitle: &str) {
    s.push_str(&format!(
        r#"<text x="{mid_x}" y="20" text-anchor="middle" fill="currentColor" font-size="13" font-weight="600">{}</text>"#,
        html_esc(label)
    ));
    s.push_str(&format!(
        r#"<text x="{mid_x}" y="37" text-anchor="middle" fill="currentColor" font-size="11" opacity="0.6">{}</text>"#,
        html_esc(subtitle)
    ));
}
