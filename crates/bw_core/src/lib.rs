//! `bw_core` — pure-Rust math for the display-bw calculator.
//!
//! * [`timing`]: CVT / CVT-RB / CVT-R2 / custom timing generation.
//! * [`link`]: data rates, link capacities, DSC fit checks, max-refresh solver.
//! * [`geometry`]: physical size, PPI, dot pitch, pixels per degree, curvature.
//!
//! All figures are approximations intended for "will it fit?" planning, not certification.

pub mod geometry;
pub mod link;
pub mod timing;

pub use geometry::*;
pub use link::*;
pub use timing::*;

#[cfg(test)]
mod tests;
