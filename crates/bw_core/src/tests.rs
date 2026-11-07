use super::*;

fn within(actual: f64, expected: f64, pct: f64) -> bool {
    ((actual - expected) / expected).abs() * 100.0 <= pct
}

fn link(id: &str) -> &'static Link {
    link_by_id(id).expect("link id")
}

fn mode(h: u32, v: u32, hz: f64, depth: u32, chroma: Chroma, standard: TimingStandard) -> Mode {
    Mode { h, v, refresh: hz, depth, chroma, standard }
}

fn fit(m: &Mode, l: &str) -> Fit {
    let t = m.timing().unwrap();
    check_fit(&t, m.depth, m.chroma, link(l), m.chroma.default_dsc_bpp()).fit
}

// ---------- timing ----------

#[test]
fn cvt_rb_1080p60() {
    let t = generate(1920, 1080, 60.0, TimingStandard::CvtRb).unwrap();
    assert!(within(t.pixel_clock_mhz, 138.5, 1.0), "{}", t.pixel_clock_mhz);
    assert_eq!((t.h_total, t.v_total), (2080, 1111));
}

#[test]
fn cvt_rb_2160p60() {
    let t = generate(3840, 2160, 60.0, TimingStandard::CvtRb).unwrap();
    assert!(within(t.pixel_clock_mhz, 533.0, 1.0), "{}", t.pixel_clock_mhz);
    assert_eq!((t.h_total, t.v_total), (4000, 2222));
}

#[test]
fn cvt_standard_1080p60() {
    let t = generate(1920, 1080, 60.0, TimingStandard::Cvt).unwrap();
    assert_eq!((t.h_total, t.v_total), (2576, 1120));
    assert!(within(t.pixel_clock_mhz, 173.0, 0.5), "{}", t.pixel_clock_mhz);
    assert!(within(t.refresh_actual, 60.0, 0.5));
}

#[test]
fn cvt_r2_is_tighter_than_rb() {
    let rb = generate(3840, 2160, 120.0, TimingStandard::CvtRb).unwrap();
    let r2 = generate(3840, 2160, 120.0, TimingStandard::CvtR2).unwrap();
    assert_eq!(r2.h_total, 3840 + 80);
    assert!(r2.pixel_clock_mhz < rb.pixel_clock_mhz);
    // R2 keeps the requested refresh almost exactly (0.001 MHz precision).
    assert!((r2.refresh_actual - 120.0).abs() < 0.01);
}

#[test]
fn vsync_aspect_fallback() {
    assert_eq!(cvt_vsync_lines(1920, 1080), 5);
    assert_eq!(cvt_vsync_lines(1920, 1200), 6);
    assert_eq!(cvt_vsync_lines(1024, 768), 4);
    assert_eq!(cvt_vsync_lines(1280, 1024), 7);
    assert_eq!(cvt_vsync_lines(3440, 1440), 10);
    assert_eq!(cvt_vsync_lines(5120, 1440), 10);
    // Non-standard aspects still produce a valid timing.
    let t = generate(5120, 1440, 240.0, TimingStandard::Cvt).unwrap();
    assert!(t.pixel_clock_mhz > 0.0);
}

#[test]
fn custom_blanking_path() {
    let t = generate(3840, 2160, 60.0, TimingStandard::Custom { h_total: 4400, v_total: 2250 }).unwrap();
    assert_eq!((t.h_total, t.v_total), (4400, 2250));
    assert!((t.pixel_clock_mhz - 594.0).abs() < 1e-9);
    assert_eq!(t.refresh_actual, 60.0);
    // 594 MHz × 24 bpp = 14.256 Gbit/s -> just fits HDMI 2.0 (14.4).
    let r = check_fit(&t, 8, Chroma::Rgb444, link("hdmi20"), 12.0);
    assert_eq!(r.fit, Fit::Uncompressed);
    // Totals smaller than the active area are rejected.
    assert_eq!(
        generate(3840, 2160, 60.0, TimingStandard::Custom { h_total: 3000, v_total: 2250 }),
        Err(TimingError::BadTotals)
    );
}

#[test]
fn invalid_inputs() {
    assert_eq!(generate(0, 1080, 60.0, TimingStandard::CvtRb), Err(TimingError::BadResolution));
    assert_eq!(generate(1920, 1080, 0.0, TimingStandard::CvtRb), Err(TimingError::BadRefresh));
    assert_eq!(generate(1920, 1080, f64::NAN, TimingStandard::Cvt), Err(TimingError::BadRefresh));
    assert_eq!(generate(1920, 1080, 5000.0, TimingStandard::CvtR2), Err(TimingError::BadRefresh));
}

#[test]
fn clock_increases_with_refresh() {
    for std in [TimingStandard::CvtR2, TimingStandard::CvtRb, TimingStandard::Cvt] {
        let mut prev = 0.0;
        for hz in 1..=1000 {
            let t = generate(3440, 1440, hz as f64, std).unwrap();
            assert!(t.pixel_clock_mhz >= prev, "{std:?} at {hz} Hz");
            prev = t.pixel_clock_mhz;
        }
    }
}

// ---------- data rate ----------

#[test]
fn bpp_by_chroma() {
    assert_eq!(bits_per_pixel(8, Chroma::Rgb444), 24.0);
    assert_eq!(bits_per_pixel(10, Chroma::Rgb444), 30.0);
    assert_eq!(bits_per_pixel(10, Chroma::Ycc422), 20.0);
    assert_eq!(bits_per_pixel(10, Chroma::Ycc420), 15.0);
}

#[test]
fn subsampling_halves_chroma() {
    for d in [6, 8, 10, 12] {
        let full = chroma_bits_per_pixel(d, Chroma::Rgb444);
        // 4:2:2 halves the chroma bits, 4:2:0 halves them again.
        assert_eq!(chroma_bits_per_pixel(d, Chroma::Ycc422), full / 2.0);
        assert_eq!(chroma_bits_per_pixel(d, Chroma::Ycc420), full / 4.0);
        // Total: 4:2:0 is half of 4:4:4, 4:2:2 is two thirds.
        let total = bits_per_pixel(d, Chroma::Rgb444);
        assert_eq!(bits_per_pixel(d, Chroma::Ycc420), total / 2.0);
        assert!((bits_per_pixel(d, Chroma::Ycc422) - total * 2.0 / 3.0).abs() < 1e-12);
    }
    let rgb = data_rate_gbps(1000.0, 10, Chroma::Rgb444);
    let y420 = data_rate_gbps(1000.0, 10, Chroma::Ycc420);
    assert!((y420 - rgb / 2.0).abs() < 1e-12);
}

#[test]
fn link_payloads() {
    assert!(within(link("hdmi14").payload_gbps(), 8.16, 0.01));
    assert!(within(link("hdmi20").payload_gbps(), 14.4, 0.01));
    assert!(within(link("frl48").payload_gbps(), 42.667, 0.01));
    assert!(within(link("hbr2").payload_gbps(), 17.28, 0.01));
    assert!(within(link("hbr3").payload_gbps(), 25.92, 0.01));
    assert!(within(link("uhbr20").payload_gbps(), 77.576, 0.01));
    assert!(!link("hdmi20").dsc && !link("hbr2").dsc);
    assert!(link("hbr3").dsc && link("frl48").dsc && link("uhbr10").dsc);
}

// ---------- fit checks ----------

#[test]
fn uhd60_8bit_fits_hdmi20() {
    for std in [TimingStandard::CvtRb, TimingStandard::CvtR2] {
        let m = mode(3840, 2160, 60.0, 8, Chroma::Rgb444, std);
        assert_eq!(fit(&m, "hdmi20"), Fit::Uncompressed);
    }
}

#[test]
fn uhd120_10bit_needs_hdmi21() {
    for std in [TimingStandard::CvtRb, TimingStandard::CvtR2] {
        let m = mode(3840, 2160, 120.0, 10, Chroma::Rgb444, std);
        assert_eq!(fit(&m, "hdmi20"), Fit::No, "HDMI 2.0 has no DSC");
        assert_eq!(fit(&m, "frl48"), Fit::Uncompressed);
    }
}

#[test]
fn super_ultrawide_240_10bit_needs_dsc_on_dp14() {
    for std in [TimingStandard::CvtRb, TimingStandard::CvtR2] {
        let m = mode(5120, 1440, 240.0, 10, Chroma::Rgb444, std);
        let t = m.timing().unwrap();
        let r = check_fit(&t, 10, Chroma::Rgb444, link("hbr3"), 12.0);
        assert_eq!(r.fit, Fit::Dsc, "{std:?}: {r:?}");
        assert!(r.headroom_pct < 0.0);
        assert!(r.dsc_headroom_pct.unwrap() >= 0.0);
        assert!((r.dsc_ratio.unwrap() - 2.5).abs() < 1e-12);
        // DP 1.2 cannot do DSC at all.
        assert_eq!(check_fit(&t, 10, Chroma::Rgb444, link("hbr2"), 12.0).fit, Fit::No);
    }
}

#[test]
fn dsc_target_must_be_below_source_bpp() {
    let t = generate(3840, 2160, 60.0, TimingStandard::CvtR2).unwrap();
    // 8-bit 4:2:0 is 12 bpp; a 12 bpp "compression" target is meaningless.
    let r = check_fit(&t, 8, Chroma::Ycc420, link("hbr3"), 12.0);
    assert!(r.dsc_ratio.is_none() && r.dsc_gbps.is_none());
    assert!(!dsc_applicable(link("hbr3"), 8, Chroma::Ycc420, 12.0));
    assert!(dsc_applicable(link("hbr3"), 8, Chroma::Ycc420, 8.0));
    assert!(!dsc_applicable(link("hbr3"), 10, Chroma::Rgb444, 0.0));
}

// ---------- max refresh ----------

#[test]
fn max_refresh_consistent_with_fit() {
    let cases = [
        (3840, 2160, 10, Chroma::Rgb444, TimingStandard::CvtR2),
        (3840, 2160, 8, Chroma::Ycc420, TimingStandard::CvtRb),
        (5120, 1440, 10, Chroma::Rgb444, TimingStandard::CvtR2),
        (2560, 1440, 8, Chroma::Rgb444, TimingStandard::Cvt),
        (7680, 4320, 12, Chroma::Ycc422, TimingStandard::CvtR2),
        (3440, 1440, 10, Chroma::Rgb444, TimingStandard::Custom { h_total: 3520, v_total: 1500 }),
    ];
    for (h, v, d, c, s) in cases {
        let base = mode(h, v, 60.0, d, c, s);
        for l in LINKS {
            let bpp = c.default_dsc_bpp();
            let un = max_refresh(&base, l, bpp, false);
            let ds = max_refresh(&base, l, bpp, true);
            let ok = |hz: u32, dsc: bool| {
                let m = Mode { refresh: hz as f64, ..base };
                match fit(&m, l.id) {
                    Fit::Uncompressed => true,
                    Fit::Dsc => dsc,
                    Fit::No => false,
                }
            };
            for (res, dsc) in [(un, false), (ds, true)] {
                match res {
                    Some(hz) => {
                        assert!(ok(hz, dsc), "{h}x{v} {} {hz}", l.id);
                        if hz < SOLVER_MAX_HZ {
                            assert!(!ok(hz + 1, dsc), "{h}x{v} {} {hz}+1", l.id);
                        }
                    }
                    None => assert!(!ok(1, dsc)),
                }
            }
            // DSC can only raise the ceiling.
            assert!(ds >= un, "{h}x{v} {}: {ds:?} < {un:?}", l.id);
            if !l.dsc {
                assert_eq!(un, ds);
            }
        }
    }
}

#[test]
fn max_refresh_monotonic_in_capacity() {
    let base = mode(3840, 2160, 60.0, 10, Chroma::Rgb444, TimingStandard::CvtR2);
    let order = ["hdmi14", "hdmi20", "frl24", "frl32", "frl40", "frl48"];
    let mut prev = 0;
    for id in order {
        let hz = max_refresh(&base, link(id), 12.0, false).unwrap_or(0);
        assert!(hz >= prev, "{id}");
        prev = hz;
    }
    let mut prev = 0;
    for id in ["hbr2", "hbr3", "uhbr10", "uhbr13", "uhbr20"] {
        let hz = max_refresh(&base, link(id), 12.0, false).unwrap_or(0);
        assert!(hz >= prev, "{id}");
        prev = hz;
    }
    // Lower bit depth never lowers the ceiling.
    for l in LINKS {
        let mut prev = u32::MAX;
        for d in [6, 8, 10, 12] {
            let m = Mode { depth: d, ..base };
            let hz = max_refresh(&m, l, 12.0, false).unwrap_or(0);
            assert!(hz <= prev, "{} depth {d}", l.id);
            prev = hz;
        }
    }
}

#[test]
fn max_refresh_none_and_cap() {
    // 8K 12-bit RGB over HDMI 1.4: does not fit even at 1 Hz with a huge custom blanking.
    let huge = mode(7680, 4320, 60.0, 12, Chroma::Rgb444, TimingStandard::Custom { h_total: 32768, v_total: 32768 });
    assert_eq!(max_refresh(&huge, link("hdmi14"), 12.0, false), None);
    // A tiny mode hits the search cap.
    let tiny = mode(640, 480, 60.0, 8, Chroma::Rgb444, TimingStandard::CvtR2);
    assert_eq!(max_refresh(&tiny, link("uhbr20"), 12.0, false), Some(SOLVER_MAX_HZ));
}

// ---------- geometry ----------

#[test]
fn ppi_27_qhd() {
    let g = geometry(27.0, 2560, 1440).unwrap();
    assert!(within(g.ppi, 108.8, 0.1), "{}", g.ppi);
    assert!(within(ppi(27.0, 2560, 1440), 108.8, 0.1));
    assert!(within(g.dot_pitch_mm, 0.2335, 0.5), "{}", g.dot_pitch_mm);
    assert_eq!(g.aspect, (16, 9));
}

#[test]
fn width_49_super_ultrawide() {
    let g = geometry(49.0, 5120, 1440).unwrap();
    assert!(within(g.width_cm(), 119.6, 1.0), "{}", g.width_cm());
    assert_eq!(g.aspect, (32, 9));
    // Diagonal is preserved.
    let d = (g.width_in.powi(2) + g.height_in.powi(2)).sqrt();
    assert!((d - 49.0).abs() < 1e-9);
}

#[test]
fn aspects() {
    assert_eq!(aspect_ratio(3440, 1440), (21, 9));
    assert_eq!(aspect_ratio(1920, 1200), (16, 10));
    assert_eq!(aspect_ratio(1366, 768), (16, 9));
    assert_eq!(aspect_ratio(1000, 1000), (1, 1));
    assert_eq!(aspect_ratio(1000, 700), (10, 7));
}

#[test]
fn viewing_distance_and_ppd() {
    let g = geometry(27.0, 2560, 1440).unwrap();
    // At the 1-arcmin distance, one pixel = 1/60 degree -> ~60 PPD.
    let ppd = pixels_per_degree(g.ppi, g.retina_distance_cm);
    assert!(within(ppd, 60.0, 0.1), "{ppd}");
    assert!(within(g.retina_distance_cm, 80.3, 1.0), "{}", g.retina_distance_cm);
    // PPD scales linearly with distance.
    assert!(within(pixels_per_degree(g.ppi, 120.0), 2.0 * pixels_per_degree(g.ppi, 60.0), 1e-9));
    assert!(within(horizontal_fov_deg(100.0, 50.0), 90.0, 1e-9));
}

#[test]
fn curvature_estimate() {
    let c = curvature(100.0, 1000.0).unwrap();
    // 100 cm of arc on a 100 cm radius: 1 rad.
    assert!(within(c.angle_deg, 57.2958, 0.01));
    assert!(within(c.chord_cm, 2.0 * 100.0 * 0.5f64.sin(), 1e-9));
    assert!(within(c.depth_cm, 100.0 * (1.0 - 0.5f64.cos()), 1e-9));
    // Flatter curve -> shallower depth.
    assert!(curvature(100.0, 1800.0).unwrap().depth_cm < c.depth_cm);
    assert!(curvature(100.0, 0.0).is_none());
}
