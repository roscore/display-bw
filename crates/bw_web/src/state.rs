//! Calculator state and its compact URL-fragment encoding (`#m=k:v,k:v,...`).
//!
//! The fragment never leaves the browser: it is not sent to any server.

use bw_core::{Chroma, TimingStandard};

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub h: u32,
    pub v: u32,
    pub refresh: f64,
    pub depth: u32,
    pub chroma: Chroma,
    /// "r2" | "rb" | "cvt" | "custom"
    pub timing: String,
    pub h_total: u32,
    pub v_total: u32,
    /// DSC target; `None` = per-chroma default.
    pub dsc_bpp: Option<f64>,
    pub link: String,
    pub diag: f64,
    pub dist_cm: f64,
    pub curve_mm: f64,
    pub b_h: u32,
    pub b_v: u32,
    pub b_diag: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            h: 5120,
            v: 1440,
            refresh: 240.0,
            depth: 10,
            chroma: Chroma::Rgb444,
            timing: "r2".into(),
            h_total: 5200,
            v_total: 1500,
            dsc_bpp: None,
            link: "hbr3".into(),
            diag: 49.0,
            dist_cm: 80.0,
            curve_mm: 1000.0,
            b_h: 2560,
            b_v: 1440,
            b_diag: 27.0,
        }
    }
}

pub fn chroma_from_code(s: &str) -> Option<Chroma> {
    match s {
        "444" | "rgb" => Some(Chroma::Rgb444),
        "422" => Some(Chroma::Ycc422),
        "420" => Some(Chroma::Ycc420),
        _ => None,
    }
}

impl State {
    pub fn standard(&self) -> TimingStandard {
        match self.timing.as_str() {
            "rb" => TimingStandard::CvtRb,
            "cvt" => TimingStandard::Cvt,
            "custom" => TimingStandard::Custom { h_total: self.h_total, v_total: self.v_total },
            _ => TimingStandard::CvtR2,
        }
    }

    pub fn effective_dsc_bpp(&self) -> f64 {
        self.dsc_bpp.unwrap_or_else(|| self.chroma.default_dsc_bpp())
    }

    /// Encode as the value of the `m` fragment parameter.
    pub fn to_fragment(&self) -> String {
        let mut parts = vec![
            format!("h:{}", self.h),
            format!("v:{}", self.v),
            format!("r:{}", num(self.refresh)),
            format!("d:{}", self.depth),
            format!("c:{}", self.chroma.code()),
            format!("t:{}", self.timing),
        ];
        if self.timing == "custom" {
            parts.push(format!("ht:{}", self.h_total));
            parts.push(format!("vt:{}", self.v_total));
        }
        if let Some(b) = self.dsc_bpp {
            parts.push(format!("dsc:{}", num(b)));
        }
        parts.push(format!("l:{}", self.link));
        parts.push(format!("dg:{}", num(self.diag)));
        parts.push(format!("vd:{}", num(self.dist_cm)));
        parts.push(format!("cr:{}", num(self.curve_mm)));
        parts.push(format!("bh:{}", self.b_h));
        parts.push(format!("bv:{}", self.b_v));
        parts.push(format!("bd:{}", num(self.b_diag)));
        parts.join(",")
    }

    /// Parse a full `location.hash` (with or without the leading `#`).
    /// Unknown keys and malformed values are ignored; missing keys keep defaults.
    pub fn from_hash(hash: &str) -> Option<State> {
        let hash = hash.trim_start_matches('#');
        let m = hash.split('&').find_map(|kv| kv.strip_prefix("m="))?;
        let mut s = State::default();
        for item in m.split(',') {
            let Some((k, val)) = item.split_once(':') else { continue };
            let u = || val.parse::<u32>().ok();
            let f = || val.parse::<f64>().ok().filter(|x| x.is_finite());
            match k {
                "h" => set(&mut s.h, u()),
                "v" => set(&mut s.v, u()),
                "r" => set(&mut s.refresh, f()),
                "d" => set(&mut s.depth, u().filter(|d| [6, 8, 10, 12].contains(d))),
                "c" => set(&mut s.chroma, chroma_from_code(val)),
                "t" => {
                    if ["r2", "rb", "cvt", "custom"].contains(&val) {
                        s.timing = val.to_string();
                    }
                }
                "ht" => set(&mut s.h_total, u()),
                "vt" => set(&mut s.v_total, u()),
                "dsc" => s.dsc_bpp = f().filter(|b| *b > 0.0),
                "l" => {
                    if bw_core::link_by_id(val).is_some() {
                        s.link = val.to_string();
                    }
                }
                "dg" => set(&mut s.diag, f()),
                "vd" => set(&mut s.dist_cm, f()),
                "cr" => set(&mut s.curve_mm, f()),
                "bh" => set(&mut s.b_h, u()),
                "bv" => set(&mut s.b_v, u()),
                "bd" => set(&mut s.b_diag, f()),
                _ => {}
            }
        }
        Some(s)
    }
}

fn set<T>(slot: &mut T, v: Option<T>) {
    if let Some(v) = v {
        *slot = v;
    }
}

/// Short number formatting for the fragment (no trailing ".0").
fn num(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        let s = format!("{:.3}", x);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut s = State::default();
        s.timing = "custom".into();
        s.h_total = 4400;
        s.v_total = 2250;
        s.dsc_bpp = Some(10.5);
        s.chroma = Chroma::Ycc420;
        let frag = s.to_fragment();
        assert_eq!(State::from_hash(&format!("#m={frag}")), Some(s));
    }

    #[test]
    fn junk_is_ignored() {
        let s = State::from_hash("#m=h:abc,zz:1,d:7,t:evil,c:444,r:144").unwrap();
        assert_eq!(s.h, State::default().h);
        assert_eq!(s.depth, State::default().depth);
        assert_eq!(s.timing, "r2");
        assert_eq!(s.refresh, 144.0);
        assert!(State::from_hash("#other").is_none());
    }
}
