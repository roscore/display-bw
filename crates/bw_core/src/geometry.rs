//! Monitor geometry: physical size, PPI, dot pitch, viewing distance, pixels per degree, curvature.
//!
//! Assumes square pixels, so the physical aspect equals the pixel aspect.

pub const MM_PER_IN: f64 = 25.4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    pub diagonal_in: f64,
    pub width_in: f64,
    pub height_in: f64,
    pub ppi: f64,
    /// Pixel pitch in millimetres.
    pub dot_pitch_mm: f64,
    /// Distance (cm) beyond which one pixel subtends less than 1 arcminute.
    pub retina_distance_cm: f64,
    /// Reduced aspect ratio, e.g. (16, 9). Very irregular sizes are approximated.
    pub aspect: (u32, u32),
}

impl Geometry {
    pub fn width_cm(&self) -> f64 {
        self.width_in * 2.54
    }
    pub fn height_cm(&self) -> f64 {
        self.height_in * 2.54
    }
    pub fn area_cm2(&self) -> f64 {
        self.width_cm() * self.height_cm()
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Reduce a pixel aspect to small integers. Marketing aspects are matched loosely:
/// 3440x1440 and 2560x1080 are reported as 21:9, 5120x1440 as 32:9, 1366x768 as 16:9.
pub fn aspect_ratio(h: u32, v: u32) -> (u32, u32) {
    if h == 0 || v == 0 {
        return (0, 0);
    }
    let r = h as f64 / v as f64;
    const NAMED: &[(u32, u32)] = &[(4, 3), (5, 4), (3, 2), (16, 10), (16, 9), (21, 9), (32, 9), (32, 10), (1, 1)];
    for &(a, b) in NAMED {
        if (r - a as f64 / b as f64).abs() / r < 0.03 {
            return (a, b);
        }
    }
    let g = gcd(h, v);
    (h / g, v / g)
}

/// Physical geometry from a diagonal and a pixel resolution.
pub fn geometry(diagonal_in: f64, h: u32, v: u32) -> Option<Geometry> {
    if !(diagonal_in > 0.0) || !diagonal_in.is_finite() || h == 0 || v == 0 {
        return None;
    }
    let (hf, vf) = (h as f64, v as f64);
    let diag_px = (hf * hf + vf * vf).sqrt();
    let width_in = diagonal_in * hf / diag_px;
    let height_in = diagonal_in * vf / diag_px;
    let ppi = diag_px / diagonal_in;
    let pitch_in = 1.0 / ppi;
    Some(Geometry {
        diagonal_in,
        width_in,
        height_in,
        ppi,
        dot_pitch_mm: pitch_in * MM_PER_IN,
        retina_distance_cm: retina_distance_in(ppi) * 2.54,
        aspect: aspect_ratio(h, v),
    })
}

/// Pixels per inch.
pub fn ppi(diagonal_in: f64, h: u32, v: u32) -> f64 {
    ((h as f64).powi(2) + (v as f64).powi(2)).sqrt() / diagonal_in
}

/// Distance (inches) at which one pixel subtends exactly 1 arcminute.
pub fn retina_distance_in(ppi: f64) -> f64 {
    let one_arcmin = (1.0f64 / 60.0).to_radians();
    (1.0 / ppi) / one_arcmin.tan()
}

/// Pixels per degree at the screen centre for a viewing distance in centimetres.
pub fn pixels_per_degree(ppi: f64, distance_cm: f64) -> f64 {
    let d_in = distance_cm / 2.54;
    2.0 * d_in * (0.5f64).to_radians().tan() * ppi
}

/// Horizontal field of view (degrees) a flat screen of `width_cm` covers at `distance_cm`.
pub fn horizontal_fov_deg(width_cm: f64, distance_cm: f64) -> f64 {
    2.0 * (width_cm / 2.0 / distance_cm).atan().to_degrees()
}

/// Curved-panel estimate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curvature {
    /// Radius in millimetres (1800R = 1800 mm).
    pub radius_mm: f64,
    /// Panel width measured along the arc, cm (assumed equal to the flat-equivalent width).
    pub arc_cm: f64,
    /// Straight-line distance between the left and right edges, cm.
    pub chord_cm: f64,
    /// Depth of the arc (sagitta): how far the centre sits behind the edges, cm.
    pub depth_cm: f64,
    /// Angle subtended by the panel at the centre of curvature, degrees.
    pub angle_deg: f64,
}

/// Estimate arc depth for a curved panel whose (arc) width is `width_cm`.
pub fn curvature(width_cm: f64, radius_mm: f64) -> Option<Curvature> {
    if !(radius_mm > 0.0) || !(width_cm > 0.0) {
        return None;
    }
    let r_cm = radius_mm / 10.0;
    let theta = width_cm / r_cm; // radians
    if theta >= 2.0 * core::f64::consts::PI {
        return None;
    }
    let chord = 2.0 * r_cm * (theta / 2.0).sin();
    let depth = r_cm * (1.0 - (theta / 2.0).cos());
    Some(Curvature {
        radius_mm,
        arc_cm: width_cm,
        chord_cm: chord,
        depth_cm: depth,
        angle_deg: theta.to_degrees(),
    })
}
