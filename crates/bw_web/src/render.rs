//! Turns a [`State`] into HTML fragments and a plain-text summary.
//!
//! Only numbers and fixed strings are interpolated, never free user text.

use crate::i18n::{t, Lang};
use crate::state::State;
use bw_core::{
    bits_per_pixel, check_fit, curvature, data_rate_gbps, dsc_applicable, geometry, horizontal_fov_deg, link_by_id,
    max_refresh, pixels_per_degree, Fit, Geometry, Mode, LINKS,
};

#[derive(Default)]
pub struct Output {
    pub error: Option<String>,
    pub timing_html: String,
    pub links_html: String,
    pub max_html: String,
    pub geo_a_html: String,
    pub geo_b_html: String,
    pub text: String,
}

fn kv(out: &mut String, k: &str, v: &str) {
    out.push_str("<dt>");
    out.push_str(k);
    out.push_str("</dt><dd>");
    out.push_str(v);
    out.push_str("</dd>");
}

fn chroma_label(s: &State) -> &'static str {
    match s.chroma {
        bw_core::Chroma::Rgb444 => "RGB/4:4:4",
        bw_core::Chroma::Ycc422 => "4:2:2",
        bw_core::Chroma::Ycc420 => "4:2:0",
    }
}

fn timing_label(s: &State) -> &'static str {
    match s.timing.as_str() {
        "rb" => "CVT-RB",
        "cvt" => "CVT",
        "custom" => "custom",
        _ => "CVT-R2",
    }
}

fn hz(v: Option<u32>, lang: Lang) -> String {
    match v {
        Some(h) if h >= bw_core::SOLVER_MAX_HZ => format!("≥ {h} Hz"),
        Some(h) => format!("{h} Hz"),
        None => t(lang, "none").to_string(),
    }
}

fn badge(class: &str, text: &str) -> String {
    format!("<span class=\"badge {class}\">{text}</span>")
}

fn pct(x: f64) -> String {
    format!("{:+.1} %", x)
}

pub fn render(s: &State, lang: Lang) -> Output {
    let mut o = Output::default();
    let mut text = String::new();
    let dsc_bpp = s.effective_dsc_bpp();
    let mode = Mode { h: s.h, v: s.v, refresh: s.refresh, depth: s.depth, chroma: s.chroma, standard: s.standard() };

    text.push_str(&format!(
        "display-bw — {}x{} @ {} Hz, {}-bit {}, {}\n",
        s.h,
        s.v,
        fmt(s.refresh, 3),
        s.depth,
        chroma_label(s),
        timing_label(s)
    ));

    match mode.timing() {
        Err(e) => {
            o.error = Some(format!("{}: {e}", t(lang, "err")));
            text.push_str(&format!("{}: {e}\n", t(lang, "err")));
        }
        Ok(tm) => {
            let bpp = bits_per_pixel(s.depth, s.chroma);
            let rate = data_rate_gbps(tm.pixel_clock_mhz, s.depth, s.chroma);
            let mut h = String::new();
            kv(&mut h, t(lang, "pixel_clock"), &format!("<b>{} MHz</b>", fmt(tm.pixel_clock_mhz, 3)));
            kv(&mut h, t(lang, "totals"), &format!("{} × {}", tm.h_total, tm.v_total));
            kv(
                &mut h,
                t(lang, "active_total"),
                &format!("{} / {} ({:.1} %)", group(tm.active_pixels()), group(tm.total_pixels()), tm.active_ratio() * 100.0),
            );
            kv(&mut h, t(lang, "blanking"), &format!("{:.1} %", (1.0 / tm.active_ratio() - 1.0) * 100.0));
            kv(&mut h, t(lang, "actual_refresh"), &format!("{} Hz", fmt(tm.refresh_actual, 3)));
            kv(&mut h, t(lang, "h_freq"), &format!("{} kHz", fmt(tm.h_freq_khz(), 2)));
            kv(&mut h, t(lang, "bpp"), &format!("{} bpp", fmt(bpp, 2)));
            kv(&mut h, t(lang, "data_rate"), &format!("<b>{} Gbit/s</b>", fmt(rate, 2)));
            o.timing_html = h;

            text.push_str(&format!(
                "{}: {} MHz ({} {}x{}, {} {} Hz)\n{}: {} Gbit/s ({} bpp); DSC target {} bpp\n\n",
                t(lang, "pixel_clock"),
                fmt(tm.pixel_clock_mhz, 3),
                t(lang, "totals"),
                tm.h_total,
                tm.v_total,
                t(lang, "actual_refresh"),
                fmt(tm.refresh_actual, 3),
                t(lang, "data_rate"),
                fmt(rate, 2),
                fmt(bpp, 2),
                fmt(dsc_bpp, 2),
            ));

            // Link table.
            let mut tb = format!(
                "<thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
                t(lang, "col_link"),
                t(lang, "col_payload"),
                t(lang, "col_uncompressed"),
                t(lang, "col_dsc"),
                t(lang, "col_max"),
            );
            for l in LINKS {
                let r = check_fit(&tm, s.depth, s.chroma, l, dsc_bpp);
                let raw_cell = if r.fit == Fit::Uncompressed {
                    format!("{} <span class=\"dim\">{} {}</span>", badge("ok", t(lang, "fit_yes")), t(lang, "headroom"), pct(r.headroom_pct))
                } else {
                    format!("{} <span class=\"dim\">{}</span>", badge("no", t(lang, "fit_no")), pct(r.headroom_pct))
                };
                let dsc_cell = if !l.dsc {
                    format!("<span class=\"dim\">{}</span>", t(lang, "dsc_unsupported"))
                } else if !dsc_applicable(l, s.depth, s.chroma, dsc_bpp) {
                    format!("<span class=\"dim\">{}</span>", t(lang, "dsc_na"))
                } else {
                    let head = r.dsc_headroom_pct.unwrap_or(f64::NEG_INFINITY);
                    let ratio = r.dsc_ratio.unwrap_or(1.0);
                    let b = if head >= 0.0 {
                        if r.fit == Fit::Uncompressed {
                            badge("ok", t(lang, "fit_yes"))
                        } else {
                            badge("dsc", t(lang, "fit_dsc"))
                        }
                    } else {
                        badge("no", t(lang, "fit_no"))
                    };
                    format!(
                        "{b} <span class=\"dim\">{} Gbit/s, {} {:.2}:1, {}</span>",
                        fmt(r.dsc_gbps.unwrap_or(0.0), 2),
                        t(lang, "ratio"),
                        ratio,
                        pct(head)
                    )
                };
                let mx_raw = max_refresh(&mode, l, dsc_bpp, false);
                let mx_dsc = max_refresh(&mode, l, dsc_bpp, true);
                let mx = if l.dsc { format!("{} / {}", hz(mx_raw, lang), hz(mx_dsc, lang)) } else { hz(mx_raw, lang) };
                tb.push_str(&format!(
                    "<tr><td>{}<br><span class=\"dim\">{} Gbit/s, {}</span></td><td>{} Gbit/s</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    l.name,
                    fmt(l.raw_gbps, 1),
                    l.coding.label(),
                    fmt(r.payload_gbps, 2),
                    raw_cell,
                    dsc_cell,
                    mx
                ));
                let verdict = match r.fit {
                    Fit::Uncompressed => format!("{} ({})", t(lang, "fit_yes"), pct(r.headroom_pct)),
                    Fit::Dsc => format!(
                        "{} {:.2}:1 ({})",
                        t(lang, "fit_dsc"),
                        r.dsc_ratio.unwrap_or(1.0),
                        pct(r.dsc_headroom_pct.unwrap_or(0.0))
                    ),
                    Fit::No => t(lang, "fit_no").to_string(),
                };
                text.push_str(&format!("{:<22} {:>6} Gbit/s  {}\n", l.name, fmt(r.payload_gbps, 2), verdict));
            }
            tb.push_str("</tbody>");
            o.links_html = tb;

            // Max refresh finder.
            if let Some(l) = link_by_id(&s.link) {
                let mut m = String::new();
                let raw = max_refresh(&mode, l, dsc_bpp, false);
                kv(&mut m, t(lang, "max_raw"), &format!("<b>{}</b>", hz(raw, lang)));
                let dsc_txt = if l.dsc && dsc_applicable(l, s.depth, s.chroma, dsc_bpp) {
                    let d = max_refresh(&mode, l, dsc_bpp, true);
                    format!("<b>{}</b> <span class=\"dim\">@ {} bpp</span>", hz(d, lang), fmt(dsc_bpp, 2))
                } else if l.dsc {
                    t(lang, "dsc_na").to_string()
                } else {
                    t(lang, "dsc_unsupported").to_string()
                };
                kv(&mut m, t(lang, "max_dsc"), &dsc_txt);
                o.max_html = m;
                text.push_str(&format!(
                    "\n{} ({}): {}",
                    t(lang, "sec_max"),
                    l.name,
                    hz(raw, lang)
                ));
                if l.dsc && dsc_applicable(l, s.depth, s.chroma, dsc_bpp) {
                    text.push_str(&format!(" / DSC {}", hz(max_refresh(&mode, l, dsc_bpp, true), lang)));
                }
                text.push('\n');
            }
        }
    }

    // Geometry.
    let ga = geometry(s.diag, s.h, s.v);
    let gb = geometry(s.b_diag, s.b_h, s.b_v);
    match ga {
        Some(g) => {
            o.geo_a_html = geo_html(&g, s, lang, true);
            text.push_str(&format!("\nA: {}\n", geo_text(&g, s.h, s.v, Some(s.dist_cm), lang)));
            if let Some(c) = curvature(g.width_cm(), s.curve_mm).filter(|_| s.curve_mm > 0.0) {
                text.push_str(&format!(
                    "   {}R: {} {:.1} cm, {} {:.1} cm\n",
                    fmt(c.radius_mm, 0),
                    t(lang, "curve_depth"),
                    c.depth_cm,
                    t(lang, "curve_chord"),
                    c.chord_cm
                ));
            }
        }
        None => o.geo_a_html = format!("<dt>{}</dt><dd></dd>", t(lang, "err")),
    }
    match gb {
        Some(g) => {
            let mut h = geo_html(&g, s, lang, false);
            if let Some(a) = ga {
                kv(&mut h, t(lang, "area_ratio"), &format!("{:.2}×", g.area_cm2() / a.area_cm2()));
            }
            o.geo_b_html = h;
            text.push_str(&format!("B: {}\n", geo_text(&g, s.b_h, s.b_v, Some(s.dist_cm), lang)));
        }
        None => o.geo_b_html = format!("<dt>{}</dt><dd></dd>", t(lang, "err")),
    }
    text.push_str(&format!("\n{}\n", t(lang, "approx")));
    o.text = text;
    o
}

fn geo_html(g: &Geometry, s: &State, lang: Lang, primary: bool) -> String {
    let mut h = String::new();
    kv(&mut h, t(lang, "aspect"), &format!("{}:{}", g.aspect.0, g.aspect.1));
    kv(
        &mut h,
        t(lang, "size"),
        &format!(
            "{:.1} × {:.1} cm <span class=\"dim\">({:.1} × {:.1} in)</span>",
            g.width_cm(),
            g.height_cm(),
            g.width_in,
            g.height_in
        ),
    );
    kv(&mut h, t(lang, "ppi"), &format!("<b>{:.1} PPI</b>", g.ppi));
    kv(&mut h, t(lang, "pitch"), &format!("{:.4} mm", g.dot_pitch_mm));
    kv(
        &mut h,
        t(lang, "retina"),
        &format!(
            "{:.0} cm <span class=\"dim\">({:.0} in) — {}</span>",
            g.retina_distance_cm,
            g.retina_distance_cm / 2.54,
            t(lang, "retina_hint")
        ),
    );
    if s.dist_cm > 0.0 {
        kv(
            &mut h,
            t(lang, "ppd"),
            &format!("<b>{:.1}</b> <span class=\"dim\">@ {} cm</span>", pixels_per_degree(g.ppi, s.dist_cm), fmt(s.dist_cm, 0)),
        );
        kv(&mut h, t(lang, "fov"), &format!("{:.1}°", horizontal_fov_deg(g.width_cm(), s.dist_cm)));
    }
    if primary {
        match curvature(g.width_cm(), s.curve_mm).filter(|_| s.curve_mm > 0.0) {
            Some(c) => {
                kv(&mut h, t(lang, "curve_depth"), &format!("{:.1} cm <span class=\"dim\">({}R)</span>", c.depth_cm, fmt(c.radius_mm, 0)));
                kv(&mut h, t(lang, "curve_chord"), &format!("{:.1} cm", c.chord_cm));
                kv(&mut h, t(lang, "curve_angle"), &format!("{:.1}°", c.angle_deg));
            }
            None => kv(&mut h, t(lang, "curve"), t(lang, "flat")),
        }
    }
    h
}

fn geo_text(g: &Geometry, hpx: u32, vpx: u32, dist: Option<f64>, lang: Lang) -> String {
    let mut s = format!(
        "{}\" {}x{} ({}:{}) {:.1} x {:.1} cm, {:.1} PPI, {:.4} mm, {} {:.0} cm",
        fmt(g.diagonal_in, 1),
        hpx,
        vpx,
        g.aspect.0,
        g.aspect.1,
        g.width_cm(),
        g.height_cm(),
        g.ppi,
        g.dot_pitch_mm,
        t(lang, "retina"),
        g.retina_distance_cm
    );
    if let Some(d) = dist.filter(|d| *d > 0.0) {
        s.push_str(&format!(", {:.1} PPD @ {} cm", pixels_per_degree(g.ppi, d), fmt(d, 0)));
    }
    s
}

/// Fixed decimals with trailing zeros trimmed.
fn fmt(x: f64, decimals: usize) -> String {
    let s = format!("{:.*}", decimals, x);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Thousands separators with a thin space.
fn group(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('\u{2009}');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_default_state() {
        let o = render(&State::default(), Lang::En);
        assert!(o.error.is_none());
        assert!(o.links_html.contains("DP 1.4"));
        assert!(o.text.contains("MHz"));
        let o = render(&State::default(), Lang::Ko);
        assert!(o.text.contains("픽셀 클럭"));
    }

    #[test]
    fn renders_errors() {
        let s = State { h: 0, diag: 0.0, ..State::default() };
        let o = render(&s, Lang::En);
        assert!(o.error.is_some());
    }

    #[test]
    fn formatting() {
        assert_eq!(fmt(138.5, 3), "138.5");
        assert_eq!(fmt(60.0, 3), "60");
        assert_eq!(group(8294400), "8\u{2009}294\u{2009}400");
    }
}
