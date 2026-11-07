//! Pixel formats, data rates, link capacities, DSC fit checks and a max-refresh solver.

use crate::timing::{generate, Timing, TimingError, TimingStandard};

/// Chroma subsampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chroma {
    /// RGB or YCbCr 4:4:4 — three full samples per pixel.
    Rgb444,
    /// YCbCr 4:2:2 — chroma halved horizontally.
    Ycc422,
    /// YCbCr 4:2:0 — chroma halved horizontally and vertically.
    Ycc420,
}

impl Chroma {
    pub fn code(&self) -> &'static str {
        match self {
            Chroma::Rgb444 => "444",
            Chroma::Ycc422 => "422",
            Chroma::Ycc420 => "420",
        }
    }

    /// Chroma samples per pixel (both Cb and Cr together).
    pub fn chroma_samples_per_pixel(&self) -> f64 {
        match self {
            Chroma::Rgb444 => 2.0,
            Chroma::Ycc422 => 1.0,
            Chroma::Ycc420 => 0.5,
        }
    }

    /// Total samples per pixel: one luma (or G) sample plus chroma.
    pub fn samples_per_pixel(&self) -> f64 {
        1.0 + self.chroma_samples_per_pixel()
    }

    /// Default DSC target in bits per pixel: 12 bpp for RGB, scaled for subsampled modes.
    pub fn default_dsc_bpp(&self) -> f64 {
        match self {
            Chroma::Rgb444 => 12.0,
            Chroma::Ycc422 => 8.0,
            Chroma::Ycc420 => 6.0,
        }
    }
}

/// Chroma bits carried per pixel for a given bit depth per channel.
pub fn chroma_bits_per_pixel(depth: u32, chroma: Chroma) -> f64 {
    depth as f64 * chroma.chroma_samples_per_pixel()
}

/// Uncompressed bits per pixel for a given bit depth per channel.
pub fn bits_per_pixel(depth: u32, chroma: Chroma) -> f64 {
    depth as f64 * chroma.samples_per_pixel()
}

/// Uncompressed stream data rate in Gbit/s: pixel clock × bpp.
///
/// Blanking time is included because it occupies link time even though it carries no pixels.
pub fn data_rate_gbps(pixel_clock_mhz: f64, depth: u32, chroma: Chroma) -> f64 {
    pixel_clock_mhz * bits_per_pixel(depth, chroma) / 1000.0
}

/// Line coding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coding {
    B8b10b,
    B16b18b,
    B128b132b,
}

impl Coding {
    pub fn efficiency(&self) -> f64 {
        match self {
            Coding::B8b10b => 8.0 / 10.0,
            Coding::B16b18b => 16.0 / 18.0,
            Coding::B128b132b => 128.0 / 132.0,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Coding::B8b10b => "8b/10b",
            Coding::B16b18b => "16b/18b",
            Coding::B128b132b => "128b/132b",
        }
    }
}

/// Approximate FEC overhead applied to 8b/10b DisplayPort when DSC is enabled
/// (DSC on DP 1.4 requires FEC, roughly 2.4 % of link bandwidth).
pub const DP_8B10B_FEC_FACTOR: f64 = 0.976;

/// A physical link.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    /// Stable short id (used in URL state).
    pub id: &'static str,
    pub name: &'static str,
    /// Raw aggregate line rate in Gbit/s (all lanes).
    pub raw_gbps: f64,
    pub coding: Coding,
    pub dsc: bool,
    /// Whether DSC on this link needs extra FEC overhead (8b/10b DP).
    pub dsc_needs_fec: bool,
}

impl Link {
    /// Effective payload after line coding, Gbit/s.
    pub fn payload_gbps(&self) -> f64 {
        self.raw_gbps * self.coding.efficiency()
    }
    /// Payload available when DSC is active (after FEC, where applicable).
    pub fn dsc_payload_gbps(&self) -> f64 {
        if self.dsc_needs_fec {
            self.payload_gbps() * DP_8B10B_FEC_FACTOR
        } else {
            self.payload_gbps()
        }
    }
}

/// Common links, ordered roughly by capacity within each family.
pub const LINKS: &[Link] = &[
    Link { id: "hdmi14", name: "HDMI 1.4 (TMDS 10.2G)", raw_gbps: 10.2, coding: Coding::B8b10b, dsc: false, dsc_needs_fec: false },
    Link { id: "hdmi20", name: "HDMI 2.0 (TMDS 18G)", raw_gbps: 18.0, coding: Coding::B8b10b, dsc: false, dsc_needs_fec: false },
    Link { id: "frl24", name: "HDMI 2.1 FRL 24G", raw_gbps: 24.0, coding: Coding::B16b18b, dsc: true, dsc_needs_fec: false },
    Link { id: "frl32", name: "HDMI 2.1 FRL 32G", raw_gbps: 32.0, coding: Coding::B16b18b, dsc: true, dsc_needs_fec: false },
    Link { id: "frl40", name: "HDMI 2.1 FRL 40G", raw_gbps: 40.0, coding: Coding::B16b18b, dsc: true, dsc_needs_fec: false },
    Link { id: "frl48", name: "HDMI 2.1 FRL 48G", raw_gbps: 48.0, coding: Coding::B16b18b, dsc: true, dsc_needs_fec: false },
    Link { id: "hbr2", name: "DP 1.2 HBR2 ×4", raw_gbps: 21.6, coding: Coding::B8b10b, dsc: false, dsc_needs_fec: false },
    Link { id: "hbr3", name: "DP 1.4 HBR3 ×4", raw_gbps: 32.4, coding: Coding::B8b10b, dsc: true, dsc_needs_fec: true },
    Link { id: "uhbr10", name: "DP 2.1 UHBR10 ×4", raw_gbps: 40.0, coding: Coding::B128b132b, dsc: true, dsc_needs_fec: false },
    Link { id: "uhbr13", name: "DP 2.1 UHBR13.5 ×4", raw_gbps: 54.0, coding: Coding::B128b132b, dsc: true, dsc_needs_fec: false },
    Link { id: "uhbr20", name: "DP 2.1 UHBR20 ×4", raw_gbps: 80.0, coding: Coding::B128b132b, dsc: true, dsc_needs_fec: false },
];

/// Look up a link by id.
pub fn link_by_id(id: &str) -> Option<&'static Link> {
    LINKS.iter().find(|l| l.id == id)
}

/// Outcome of a fit check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// Fits without compression.
    Uncompressed,
    /// Fits only with DSC at the selected target.
    Dsc,
    /// Does not fit.
    No,
}

/// Full fit report for one link.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FitReport {
    pub fit: Fit,
    /// Required uncompressed rate, Gbit/s.
    pub required_gbps: f64,
    /// Effective payload, Gbit/s.
    pub payload_gbps: f64,
    /// Headroom for the uncompressed stream, percent of payload (negative = over).
    pub headroom_pct: f64,
    /// DSC rate at the target bpp, if DSC is supported and meaningful.
    pub dsc_gbps: Option<f64>,
    /// Headroom with DSC, percent of DSC payload.
    pub dsc_headroom_pct: Option<f64>,
    /// Compression ratio (uncompressed bpp / target bpp), if DSC applies.
    pub dsc_ratio: Option<f64>,
}

/// DSC target bpp is only meaningful when it is positive and below the uncompressed bpp.
pub fn dsc_applicable(link: &Link, depth: u32, chroma: Chroma, dsc_bpp: f64) -> bool {
    link.dsc && dsc_bpp.is_finite() && dsc_bpp > 0.0 && dsc_bpp < bits_per_pixel(depth, chroma)
}

fn headroom(payload: f64, need: f64) -> f64 {
    (payload - need) / payload * 100.0
}

/// Check a timing against a link.
pub fn check_fit(t: &Timing, depth: u32, chroma: Chroma, link: &Link, dsc_bpp: f64) -> FitReport {
    let need = data_rate_gbps(t.pixel_clock_mhz, depth, chroma);
    let payload = link.payload_gbps();
    let (dsc_gbps, dsc_head, ratio) = if dsc_applicable(link, depth, chroma, dsc_bpp) {
        let r = t.pixel_clock_mhz * dsc_bpp / 1000.0;
        let p = link.dsc_payload_gbps();
        (Some(r), Some(headroom(p, r)), Some(bits_per_pixel(depth, chroma) / dsc_bpp))
    } else {
        (None, None, None)
    };
    let fit = if need <= payload {
        Fit::Uncompressed
    } else if dsc_head.map_or(false, |h| h >= 0.0) {
        Fit::Dsc
    } else {
        Fit::No
    };
    FitReport {
        fit,
        required_gbps: need,
        payload_gbps: payload,
        headroom_pct: headroom(payload, need),
        dsc_gbps,
        dsc_headroom_pct: dsc_head,
        dsc_ratio: ratio,
    }
}

/// A complete display mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mode {
    pub h: u32,
    pub v: u32,
    pub refresh: f64,
    pub depth: u32,
    pub chroma: Chroma,
    pub standard: TimingStandard,
}

impl Mode {
    pub fn timing(&self) -> Result<Timing, TimingError> {
        generate(self.h, self.v, self.refresh, self.standard)
    }
}

/// Upper bound for the max-refresh search (Hz).
pub const SOLVER_MAX_HZ: u32 = 1000;

/// Highest integer refresh rate (1..=SOLVER_MAX_HZ) at which `mode` (refresh ignored) fits
/// `link`, uncompressed (`with_dsc == false`) or allowing DSC at `dsc_bpp`.
///
/// Returns `None` if even 1 Hz does not fit. The required bandwidth is non-decreasing in
/// refresh rate for every supported timing formula, so a binary search is exact; the final
/// linear guard keeps the result consistent with [`check_fit`] even at rounding edges.
pub fn max_refresh(mode: &Mode, link: &Link, dsc_bpp: f64, with_dsc: bool) -> Option<u32> {
    let fits = |hz: u32| -> bool {
        let m = Mode { refresh: hz as f64, ..*mode };
        match m.timing() {
            Ok(t) => {
                let r = check_fit(&t, m.depth, m.chroma, link, dsc_bpp);
                match r.fit {
                    Fit::Uncompressed => true,
                    Fit::Dsc => with_dsc,
                    Fit::No => false,
                }
            }
            Err(_) => false,
        }
    };
    if !fits(1) {
        return None;
    }
    let (mut lo, mut hi) = (1u32, SOLVER_MAX_HZ);
    if fits(hi) {
        return Some(hi);
    }
    // invariant: fits(lo) && !fits(hi)
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if fits(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    while lo > 1 && !fits(lo) {
        lo -= 1;
    }
    Some(lo)
}
