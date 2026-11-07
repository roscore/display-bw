//! Browser UI for display-bw. Plain `web-sys`, no framework.
//!
//! Every input change re-reads the form, recomputes everything with `bw_core`, re-renders and
//! writes the state into the URL fragment with `history.replaceState` (no network, no history spam).

mod i18n;
mod render;
mod state;

use i18n::{t, Lang};
use state::State;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Event, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Window};

const LANG_KEY: &str = "display-bw.lang";

fn window() -> Window {
    web_sys::window().expect("window")
}
fn document() -> Document {
    window().document().expect("document")
}
fn el(id: &str) -> Option<Element> {
    document().get_element_by_id(id)
}
fn input(id: &str) -> Option<HtmlInputElement> {
    el(id)?.dyn_into().ok()
}
fn select(id: &str) -> Option<HtmlSelectElement> {
    el(id)?.dyn_into().ok()
}
fn input_val(id: &str) -> String {
    input(id).map(|i| i.value()).unwrap_or_default()
}
fn select_val(id: &str) -> String {
    select(id).map(|s| s.value()).unwrap_or_default()
}
fn set_input(id: &str, v: &str) {
    if let Some(i) = input(id) {
        i.set_value(v);
    }
}
fn set_select(id: &str, v: &str) {
    if let Some(s) = select(id) {
        s.set_value(v);
    }
}
fn set_html(id: &str, html: &str) {
    if let Some(e) = el(id) {
        e.set_inner_html(html);
    }
}

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

fn current_lang() -> Lang {
    let root = document().document_element();
    root.and_then(|r| r.get_attribute("lang"))
        .and_then(|l| Lang::from_code(&l))
        .unwrap_or(Lang::En)
}

fn initial_lang() -> Lang {
    if let Some(l) = storage().and_then(|s| s.get_item(LANG_KEY).ok().flatten()).and_then(|l| Lang::from_code(&l)) {
        return l;
    }
    let nav = window().navigator();
    let mut cands: Vec<String> = Vec::new();
    for v in nav.languages().iter() {
        if let Some(s) = v.as_string() {
            cands.push(s);
        }
    }
    if let Some(l) = nav.language() {
        cands.push(l);
    }
    // Default follows the browser: Korean if the first recognised language is Korean.
    cands.iter().find_map(|c| Lang::from_code(c)).unwrap_or(Lang::En)
}

fn apply_lang(lang: Lang) {
    let doc = document();
    if let Some(root) = doc.document_element() {
        let _ = root.set_attribute("lang", lang.code());
    }
    if let Ok(nodes) = doc.query_selector_all("[data-i18n]") {
        for i in 0..nodes.length() {
            if let Some(e) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                if let Some(k) = e.get_attribute("data-i18n") {
                    e.set_text_content(Some(t(lang, &k)));
                }
            }
        }
    }
    if let Some(b) = el("lang") {
        b.set_text_content(Some(t(lang, "lang_toggle")));
    }
    set_html("notes", t(lang, "notes_html"));
    doc.set_title(&format!("display-bw — {}", t(lang, "title")));
    // Link names are language-neutral, but rebuild the select in case it is empty.
    fill_link_select();
}

fn fill_link_select() {
    let Some(sel) = select("link") else { return };
    if sel.length() > 0 {
        return;
    }
    let html: String = bw_core::LINKS
        .iter()
        .map(|l| format!("<option value=\"{}\">{}</option>", l.id, l.name))
        .collect();
    sel.set_inner_html(&html);
}

fn parse_res(s: &str) -> Option<(u32, u32)> {
    let (a, b) = s.split_once('x')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

fn sync_preset(preset_id: &str, h: u32, v: u32) {
    let key = format!("{h}x{v}");
    let Some(sel) = select(preset_id) else { return };
    let has = (0..sel.length()).any(|i| {
        sel.item(i)
            .and_then(|o| o.dyn_into::<web_sys::HtmlOptionElement>().ok())
            .map_or(false, |o| o.value() == key)
    });
    sel.set_value(if has { &key } else { "custom" });
}

fn sync_hz_preset(hz: f64) {
    let presets = [60.0, 120.0, 144.0, 165.0, 240.0, 360.0, 480.0];
    let v = if presets.contains(&hz) { format!("{}", hz as u32) } else { "custom".to_string() };
    set_select("hz-preset", &v);
}

/// Push a state into the form controls.
fn write_form(s: &State) {
    set_input("h", &s.h.to_string());
    set_input("v", &s.v.to_string());
    sync_preset("res-preset", s.h, s.v);
    set_input("hz", &fmt_num(s.refresh));
    sync_hz_preset(s.refresh);
    set_select("depth", &s.depth.to_string());
    set_select("chroma", s.chroma.code());
    set_select("timing", &s.timing);
    set_input("ht", &s.h_total.to_string());
    set_input("vt", &s.v_total.to_string());
    set_input("dsc", &s.dsc_bpp.map(fmt_num).unwrap_or_default());
    set_select("link", &s.link);
    set_input("diag", &fmt_num(s.diag));
    set_input("dist", &fmt_num(s.dist_cm));
    set_input("curve", &fmt_num(s.curve_mm));
    set_input("b-h", &s.b_h.to_string());
    set_input("b-v", &s.b_v.to_string());
    sync_preset("b-preset", s.b_h, s.b_v);
    set_input("b-diag", &fmt_num(s.b_diag));
}

fn fmt_num(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{}", x as i64)
    } else {
        let s = format!("{:.4}", x);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Read the form into a state. Unparsable fields fall back to defaults/zero (validated later).
fn read_form() -> State {
    let d = State::default();
    let u = |id: &str| input_val(id).trim().parse::<u32>().unwrap_or(0);
    let f = |id: &str| input_val(id).trim().parse::<f64>().ok().filter(|x| x.is_finite());
    State {
        h: u("h"),
        v: u("v"),
        refresh: f("hz").unwrap_or(0.0),
        depth: select_val("depth").parse().unwrap_or(d.depth),
        chroma: state::chroma_from_code(&select_val("chroma")).unwrap_or(d.chroma),
        timing: {
            let tv = select_val("timing");
            if tv.is_empty() { d.timing.clone() } else { tv }
        },
        h_total: u("ht"),
        v_total: u("vt"),
        dsc_bpp: f("dsc").filter(|b| *b > 0.0),
        link: {
            let l = select_val("link");
            if bw_core::link_by_id(&l).is_some() { l } else { d.link.clone() }
        },
        diag: f("diag").unwrap_or(0.0),
        dist_cm: f("dist").unwrap_or(0.0),
        curve_mm: f("curve").unwrap_or(0.0),
        b_h: u("b-h"),
        b_v: u("b-v"),
        b_diag: f("b-diag").unwrap_or(0.0),
    }
}

fn update() {
    let s = read_form();
    let lang = current_lang();

    if let Some(ct) = el("custom-totals") {
        let style = if s.timing == "custom" { "" } else { "display:none" };
        let _ = ct.set_attribute("style", style);
    }
    if let Some(dsc) = input("dsc") {
        dsc.set_placeholder(&format!("{} ({})", fmt_num(s.chroma.default_dsc_bpp()), t(lang, "dsc_auto")));
    }

    let out = render::render(&s, lang);
    match &out.error {
        Some(e) => {
            set_html("mode-err", e);
            if let Some(x) = el("mode-err") {
                let _ = x.remove_attribute("hidden");
            }
        }
        None => {
            if let Some(x) = el("mode-err") {
                let _ = x.set_attribute("hidden", "");
            }
        }
    }
    set_html("out-timing", &out.timing_html);
    set_html("out-links", &out.links_html);
    set_html("out-max", &out.max_html);
    set_html("out-geo-a", &out.geo_a_html);
    set_html("out-geo-b", &out.geo_b_html);
    if let Some(ta) = el("out-text").and_then(|e| e.dyn_into::<HtmlTextAreaElement>().ok()) {
        ta.set_value(&out.text);
    }
    draw_canvas(&s, lang);

    // Persist state in the fragment only (never sent to a server).
    let url = format!("#m={}", s.to_fragment());
    if let Ok(h) = window().history() {
        let _ = h.replace_state_with_url(&JsValue::NULL, "", Some(&url));
    }
}

fn prefers_dark() -> bool {
    window()
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
        .map_or(false, |m| m.matches())
}

fn draw_canvas(s: &State, lang: Lang) {
    let Some(canvas) = el("cmp").and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok()) else { return };
    let css_w = canvas.client_width().max(1) as f64;
    let css_h = canvas.client_height().max(1) as f64;
    let dpr = window().device_pixel_ratio().max(1.0);
    canvas.set_width((css_w * dpr) as u32);
    canvas.set_height((css_h * dpr) as u32);
    let Ok(Some(ctx)) = canvas.get_context("2d") else { return };
    let Ok(ctx) = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>() else { return };
    let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    ctx.clear_rect(0.0, 0.0, css_w, css_h);

    let dark = prefers_dark();
    let (ca, cb, fg) = if dark { ("#60a5fa", "#f472b6", "#e6e9ee") } else { ("#2563eb", "#db2777", "#1b1f24") };
    let ga = bw_core::geometry(s.diag, s.h, s.v);
    let gb = bw_core::geometry(s.b_diag, s.b_h, s.b_v);
    let mons: Vec<(&str, &str, bw_core::Geometry, f64)> = [(ga, ca, "A", s.diag), (gb, cb, "B", s.b_diag)]
        .into_iter()
        .filter_map(|(g, c, n, d)| g.map(|g| (n, c, g, d)))
        .collect();
    if mons.is_empty() {
        return;
    }
    // One common cm -> px scale; outlines overlaid, bottom-centre aligned, so they are comparable.
    let pad = 28.0;
    let max_w = mons.iter().map(|m| m.2.width_cm()).fold(0.0, f64::max);
    let max_h = mons.iter().map(|m| m.2.height_cm()).fold(0.0, f64::max);
    let scale = ((css_w - 2.0 * pad) / max_w).min((css_h - 2.0 * pad) / max_h);
    let base_y = css_h - pad;
    let cx = css_w / 2.0;
    ctx.set_font("12px system-ui, sans-serif");
    for (i, (name, color, g, d)) in mons.iter().enumerate() {
        let w = g.width_cm() * scale;
        let h = g.height_cm() * scale;
        let x = cx - w / 2.0;
        let y = base_y - h;
        ctx.set_stroke_style_str(color);
        ctx.set_line_width(2.0);
        if i == 1 {
            let dash = js_sys::Array::of2(&JsValue::from_f64(6.0), &JsValue::from_f64(4.0));
            let _ = ctx.set_line_dash(&dash);
        } else {
            let _ = ctx.set_line_dash(&js_sys::Array::new());
        }
        ctx.stroke_rect(x, y, w, h);
        ctx.set_fill_style_str(color);
        let label = format!("{name}: {}\" {}×{} — {:.1} × {:.1} cm", fmt_num(*d), if i == 0 { s.h } else { s.b_h }, if i == 0 { s.v } else { s.b_v }, g.width_cm(), g.height_cm());
        let ty = if i == 0 { y - 6.0 } else { y + 16.0 };
        let _ = ctx.fill_text(&label, x + 4.0, ty.max(12.0));
    }
    let _ = ctx.set_line_dash(&js_sys::Array::new());
    ctx.set_fill_style_str(fg);
    let _ = ctx.fill_text(t(lang, "compare"), 6.0, css_h - 8.0);
}

fn set_status(msg: &str) {
    if let Some(e) = el("copy-status") {
        e.set_text_content(Some(msg));
    }
}

fn copy_text(text: String) {
    let lang = current_lang();
    let clip = window().navigator().clipboard();
    let promise = clip.write_text(&text);
    let ok = Closure::once(move |_v: JsValue| set_status(t(lang, "copied")));
    let fail = Closure::once(move |_v: JsValue| {
        set_status(t(lang, "copy_failed"));
        if let Some(ta) = el("out-text").and_then(|e| e.dyn_into::<HtmlTextAreaElement>().ok()) {
            ta.select();
        }
    });
    let _ = promise.then2(&ok, &fail);
    ok.forget();
    fail.forget();
}

fn on(id: &str, ev: &str, f: impl FnMut(Event) + 'static) {
    if let Some(e) = el(id) {
        let cb = Closure::<dyn FnMut(Event)>::new(f);
        let _ = e.add_event_listener_with_callback(ev, cb.as_ref().unchecked_ref());
        cb.forget();
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    let lang = initial_lang();
    fill_link_select();
    apply_lang(lang);

    let hash = window().location().hash().unwrap_or_default();
    let s = State::from_hash(&hash).unwrap_or_default();
    write_form(&s);

    // Preset selects fill the numeric fields.
    on("res-preset", "change", |_| {
        if let Some((h, v)) = parse_res(&select_val("res-preset")) {
            set_input("h", &h.to_string());
            set_input("v", &v.to_string());
        }
        update();
    });
    on("b-preset", "change", |_| {
        if let Some((h, v)) = parse_res(&select_val("b-preset")) {
            set_input("b-h", &h.to_string());
            set_input("b-v", &v.to_string());
        }
        update();
    });
    on("hz-preset", "change", |_| {
        let v = select_val("hz-preset");
        if v != "custom" {
            set_input("hz", &v);
        }
        update();
    });
    for id in ["h", "v"] {
        on(id, "input", |_| {
            let s = read_form();
            sync_preset("res-preset", s.h, s.v);
            update();
        });
    }
    for id in ["b-h", "b-v"] {
        on(id, "input", |_| {
            let s = read_form();
            sync_preset("b-preset", s.b_h, s.b_v);
            update();
        });
    }
    on("hz", "input", |_| {
        sync_hz_preset(read_form().refresh);
        update();
    });
    on("timing", "change", |_| {
        // Seed custom totals from the current CVT-R2 timing so the user starts from something sane.
        let s = read_form();
        if s.timing == "custom" {
            if let Ok(tm) = bw_core::generate(s.h, s.v, s.refresh, bw_core::TimingStandard::CvtR2) {
                if s.h_total < s.h || s.v_total < s.v {
                    set_input("ht", &tm.h_total.to_string());
                    set_input("vt", &tm.v_total.to_string());
                }
            }
        }
        update();
    });
    for id in ["depth", "chroma", "link"] {
        on(id, "change", |_| update());
    }
    for id in ["ht", "vt", "dsc", "diag", "dist", "curve", "b-diag"] {
        on(id, "input", |_| update());
    }
    on("lang", "click", |_| {
        let l = current_lang().other();
        if let Some(st) = storage() {
            let _ = st.set_item(LANG_KEY, l.code());
        }
        apply_lang(l);
        update();
    });
    on("copy", "click", |_| copy_text(render::render(&read_form(), current_lang()).text));
    on("share", "click", |_| {
        let href = window().location().href().unwrap_or_default();
        copy_text(href);
    });

    // Redraw the canvas on resize and when the colour scheme flips.
    let cb = Closure::<dyn FnMut(Event)>::new(|_| update());
    let _ = window().add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref());
    if let Ok(Some(mq)) = window().match_media("(prefers-color-scheme: dark)") {
        let _ = mq.add_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
    }
    cb.forget();

    update();
}
