# Task: Display link bandwidth & monitor geometry calculator (Rust → WebAssembly, static web app)

Original open-source tool. Write all code yourself; do not copy anyone's code, tables or assets
(derive timing math from the public VESA CVT formulas / your own understanding).

## Product
A single-page web app (GitHub Pages, sub-path `/display-bw/`) for PC gamers and people buying
high-refresh / ultrawide / super-ultrawide monitors who ask "will this cable/port run 5120x1440 @ 240 Hz
10-bit, and do I need DSC or 4:2:2?".

1. **Mode input**: horizontal × vertical resolution (presets: 1920x1080, 2560x1440, 3440x1440,
   3840x2160, 5120x1440, 5120x2160, 7680x2160, 7680x4320), refresh rate (Hz, presets 60/120/144/165/
   240/360/480), bit depth per channel (6/8/10/12), chroma (RGB/4:4:4, 4:2:2, 4:2:0),
   timing standard: CVT-R2 (default), CVT-RB, CVT (standard blanking), or custom blanking
   (H/V total entered manually).
2. **Results**: pixel clock (MHz), active vs total pixels, uncompressed data rate (Gbit/s),
   and a table of common links with their effective payload after line coding:
   HDMI 1.4 (10.2 Gbps TMDS, 8b/10b), HDMI 2.0 (18, 8b/10b), HDMI 2.1 FRL (24/32/40/48 Gbps, 16b/18b),
   DP 1.2 HBR2 ×4, DP 1.4 HBR3 ×4 (8b/10b), DP 2.1 UHBR10/13.5/20 ×4 (128b/132b),
   Thunderbolt/USB4 DP-alt hints optional. For each: fits uncompressed? / fits with DSC (bpp target
   selectable, default 12 bpp for RGB, show the compression ratio) / no. Show headroom %.
   Clearly mark figures as approximate and explain assumptions (overheads, FEC/DSC simplifications) in a
   collapsible note. Max refresh finder: for the selected resolution/depth/link, the highest refresh rate
   that fits (uncompressed and with DSC).
3. **Geometry**: diagonal (inches) + aspect from resolution → width/height (cm & in), PPI, dot pitch (mm),
   viewing distance at which pixels become indistinguishable (1 arcmin rule), pixels-per-degree at a
   user-given viewing distance, and curvature note (1000R/1800R input → arc depth estimate, optional).
   Side-by-side comparison of two monitors (physical size outline drawn to scale on a canvas).
4. Korean + English UI toggle (default follows browser language). Mobile friendly. Dark/light follows
   system. State shareable via URL fragment (#m=...), never sent to a server. Copy-result button
   (plain text summary).

## Core crate (pure Rust, no web deps, unit tested) — `crates/bw_core`
- CVT / CVT-RB / CVT-R2 timing generation (h/v total, pixel clock), data-rate calc for each chroma and
  depth, link capacity table, DSC fit check, max-refresh solver, geometry/PPI/PPD functions.
- Tests: 1920x1080@60 CVT-RB pixel clock ≈ 138.5 MHz (within 1%), 3840x2160@60 CVT-RB ≈ 533 MHz
  (within 1%), 4K60 8-bit RGB fits HDMI 2.0 but 4K120 10-bit RGB does not; 4K120 10-bit fits HDMI 2.1
  48G; 5120x1440@240 10-bit RGB needs DSC on DP 1.4; 4:2:0 halves chroma bits; PPI of 27" 2560x1440
  ≈ 108.8; 49" 32:9 5120x1440 width ≈ 119.6 cm (within 1%); max-refresh solver monotonic and
  consistent with fit check; custom blanking path.
- `crates/bw_web`: UI. Prefer plain web-sys (or a small framework) — keep the bundle small.

## Repo hygiene
- README (Korean first, then English): what it is, how the math works & assumptions, how to use,
  local build (`cargo test -p bw_core`, `trunk build --release`), MIT license.
- LICENSE: MIT, copyright "display-bw contributors". `.gitignore` for target/ dist/.
- `index.html` with a CSP meta that blocks external requests:
  `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; object-src 'none'; base-uri 'self'`.
  Tip: in `Trunk.toml` set `filehash = false` and
  `pattern_script = "<script type=\"module\" src=\"{base}boot.js\"></script>"` with an external
  `web/boot.js` (copied via `<link data-trunk rel="copy-file">`) that imports `./bw_web.js` and calls
  init, so no inline script is needed.
- `Trunk.toml`: `public_url = "/display-bw/"`.
- `.github/workflows/pages.yml`: on push to main + workflow_dispatch; checkout, rust stable + wasm32
  target, install trunk 0.22.1 binary release, `sh tools/privacy_check.sh`,
  `cargo test -p bw_core --release`, `trunk build --release`, configure-pages@v5,
  upload-pages-artifact@v3 (dist), deploy job with actions/deploy-pages@v4
  (permissions pages: write, id-token: write).
- `tools/privacy_check.sh` already exists; keep it and keep it clean.

## Done when
- `cargo test -p bw_core` green; `trunk build --release` succeeds; `sh tools/privacy_check.sh` clean.
- Commit everything with clear commits. Do NOT create new GitHub repos or change repo settings.
