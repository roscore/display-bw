//! Video timing generation.
//!
//! Implements the public VESA Coordinated Video Timings formulas for progressive scan,
//! no margins: standard blanking (CVT), reduced blanking v1 (CVT-RB) and reduced
//! blanking v2 (CVT-R2), plus a "custom" path where H/V totals are given directly.

use core::fmt;

/// Which blanking formula to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimingStandard {
    /// CVT reduced blanking v2 (80 px horizontal blank, 0.001 MHz clock precision).
    CvtR2,
    /// CVT reduced blanking v1 (160 px horizontal blank, 0.25 MHz clock step).
    CvtRb,
    /// CVT standard (CRT-style) blanking.
    Cvt,
    /// Totals supplied by the user; pixel clock = h_total × v_total × refresh.
    Custom { h_total: u32, v_total: u32 },
}

impl TimingStandard {
    /// Short stable code used for URL state.
    pub fn code(&self) -> &'static str {
        match self {
            TimingStandard::CvtR2 => "r2",
            TimingStandard::CvtRb => "rb",
            TimingStandard::Cvt => "cvt",
            TimingStandard::Custom { .. } => "custom",
        }
    }
}

/// A generated timing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    pub h_active: u32,
    pub v_active: u32,
    pub h_total: u32,
    pub v_total: u32,
    /// Pixel clock in MHz.
    pub pixel_clock_mhz: f64,
    /// Requested refresh rate in Hz.
    pub refresh_requested: f64,
    /// Refresh rate that results from the (rounded) pixel clock, in Hz.
    pub refresh_actual: f64,
}

impl Timing {
    pub fn active_pixels(&self) -> u64 {
        self.h_active as u64 * self.v_active as u64
    }
    pub fn total_pixels(&self) -> u64 {
        self.h_total as u64 * self.v_total as u64
    }
    /// Fraction of transmitted pixels that are visible (0..=1).
    pub fn active_ratio(&self) -> f64 {
        self.active_pixels() as f64 / self.total_pixels() as f64
    }
    pub fn h_freq_khz(&self) -> f64 {
        self.pixel_clock_mhz * 1000.0 / self.h_total as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimingError {
    /// Zero/absurd resolution.
    BadResolution,
    /// Refresh rate is zero, negative, not finite or too high for the formula.
    BadRefresh,
    /// Custom totals smaller than the active area.
    BadTotals,
}

impl fmt::Display for TimingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimingError::BadResolution => write!(f, "invalid resolution"),
            TimingError::BadRefresh => write!(f, "refresh rate out of range for this timing formula"),
            TimingError::BadTotals => write!(f, "total must be at least the active size"),
        }
    }
}

/// Largest resolution accepted (per axis).
pub const MAX_ACTIVE: u32 = 32_768;
/// Largest refresh rate accepted (Hz).
pub const MAX_REFRESH: f64 = 1000.0;

const CELL: u32 = 8; // character cell granularity, pixels

/// V-sync width (lines) picked by CVT v1 from the aspect ratio.
///
/// The table only covers 4:3, 16:9, 16:10, 5:4 and 15:9. Every other aspect
/// (21:9, 32:9, ...) uses 10 lines, the value CVT reserves for "non-standard" aspects.
pub fn cvt_vsync_lines(h: u32, v: u32) -> u32 {
    let is = |a: u32, b: u32| h as u64 * b as u64 == v as u64 * a as u64;
    // Tolerate the common non-exact 16:9 sizes such as 1366x768 by also checking
    // a rounded aspect.
    let near = |a: f64, b: f64| ((h as f64 / v as f64) - a / b).abs() < 0.01;
    if is(4, 3) {
        4
    } else if is(16, 9) || near(16.0, 9.0) {
        5
    } else if is(16, 10) {
        6
    } else if is(5, 4) || is(15, 9) {
        7
    } else {
        10
    }
}

fn round_down_to(x: u32, step: u32) -> u32 {
    x / step * step
}

/// Generate a timing.
pub fn generate(
    h_active: u32,
    v_active: u32,
    refresh_hz: f64,
    standard: TimingStandard,
) -> Result<Timing, TimingError> {
    if h_active == 0 || v_active == 0 || h_active > MAX_ACTIVE || v_active > MAX_ACTIVE {
        return Err(TimingError::BadResolution);
    }
    if !refresh_hz.is_finite() || refresh_hz <= 0.0 || refresh_hz > MAX_REFRESH {
        return Err(TimingError::BadRefresh);
    }
    match standard {
        TimingStandard::Custom { h_total, v_total } => {
            if h_total < h_active || v_total < v_active {
                return Err(TimingError::BadTotals);
            }
            let clk = h_total as f64 * v_total as f64 * refresh_hz / 1e6;
            Ok(Timing {
                h_active,
                v_active,
                h_total,
                v_total,
                pixel_clock_mhz: clk,
                refresh_requested: refresh_hz,
                refresh_actual: refresh_hz,
            })
        }
        TimingStandard::CvtRb => reduced(h_active, v_active, refresh_hz, false),
        TimingStandard::CvtR2 => reduced(h_active, v_active, refresh_hz, true),
        TimingStandard::Cvt => standard_blanking(h_active, v_active, refresh_hz),
    }
}

/// CVT reduced blanking, v1 (`v2 == false`) or v2.
fn reduced(h_active: u32, v_active: u32, refresh: f64, v2: bool) -> Result<Timing, TimingError> {
    const MIN_VBLANK_US: f64 = 460.0;
    const MIN_V_BACK_PORCH: u32 = 6;

    // v1 rounds the active width to the character cell; v2 has 1-pixel granularity.
    let h_act = if v2 { h_active } else { round_down_to(h_active, CELL) };
    let (h_blank, v_front_porch_min, v_sync, clock_step) = if v2 {
        (80u32, 1u32, 8u32, 0.001f64)
    } else {
        (160, 3, cvt_vsync_lines(h_active, v_active), 0.25)
    };

    let frame_us = 1e6 / refresh;
    let h_period_est = (frame_us - MIN_VBLANK_US) / v_active as f64;
    if !(h_period_est > 0.0) {
        return Err(TimingError::BadRefresh);
    }
    let vbi_lines = (MIN_VBLANK_US / h_period_est).floor() as u32 + 1;
    let min_vbi = v_front_porch_min + v_sync + MIN_V_BACK_PORCH;
    let v_blank = vbi_lines.max(min_vbi);

    let h_total = h_act + h_blank;
    let v_total = v_active + v_blank;
    let ideal = refresh * h_total as f64 * v_total as f64 / 1e6;
    // Round down to the clock step (tiny epsilon guards against binary FP noise).
    let clk = ((ideal / clock_step) + 1e-9).floor() * clock_step;
    let actual = clk * 1e6 / (h_total as f64 * v_total as f64);
    Ok(Timing {
        h_active: h_act,
        v_active,
        h_total,
        v_total,
        pixel_clock_mhz: clk,
        refresh_requested: refresh,
        refresh_actual: actual,
    })
}

/// CVT standard (CRT) blanking.
fn standard_blanking(h_active: u32, v_active: u32, refresh: f64) -> Result<Timing, TimingError> {
    const MIN_VSYNC_BP_US: f64 = 550.0;
    const MIN_V_PORCH: u32 = 3;
    const MIN_V_BACK_PORCH: u32 = 6;
    const CLOCK_STEP: f64 = 0.25;
    // Blanking formula gradient / offset after applying the CVT scaling factors
    // (C=40, J=20, K=128, M=600 -> C'=30, M'=300).
    const C_PRIME: f64 = 30.0;
    const M_PRIME: f64 = 300.0;

    let h_act = round_down_to(h_active, CELL);
    let v_sync = cvt_vsync_lines(h_active, v_active);

    let frame_us = 1e6 / refresh;
    let h_period_est = (frame_us - MIN_VSYNC_BP_US) / (v_active + MIN_V_PORCH) as f64;
    if !(h_period_est > 0.0) {
        return Err(TimingError::BadRefresh);
    }
    let vsync_bp = ((MIN_VSYNC_BP_US / h_period_est).floor() as u32 + 1).max(v_sync + MIN_V_BACK_PORCH);
    let v_total = v_active + vsync_bp + MIN_V_PORCH;

    let duty = (C_PRIME - M_PRIME * h_period_est / 1000.0).max(20.0);
    let h_blank_raw = h_act as f64 * duty / (100.0 - duty);
    let h_blank = (h_blank_raw / (2 * CELL) as f64).floor() as u32 * 2 * CELL;
    let h_total = h_act + h_blank;

    let clk = ((h_total as f64 / h_period_est / CLOCK_STEP) + 1e-9).floor() * CLOCK_STEP;
    let actual = clk * 1e6 / (h_total as f64 * v_total as f64);
    Ok(Timing {
        h_active: h_act,
        v_active,
        h_total,
        v_total,
        pixel_clock_mhz: clk,
        refresh_requested: refresh,
        refresh_actual: actual,
    })
}
