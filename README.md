# display-bw

**[한국어](#한국어) · [English](#english)**

---

## 한국어

고주사율 · 울트라와이드 · 슈퍼 울트라와이드 모니터를 고르는 PC 게이머를 위한 **디스플레이 링크 대역폭 · 모니터 크기 계산기**입니다.
"이 케이블/포트로 5120x1440 @ 240 Hz 10비트가 나올까? DSC나 4:2:2가 필요할까?"에 답해 줍니다.

Rust → WebAssembly로 만든 정적 단일 페이지 앱이고, **모든 계산은 브라우저 안에서만** 실행됩니다.
분석 도구·추적기·외부 CDN/폰트·런타임 네트워크 요청이 없습니다 (CSP로 차단).

### 기능
- **모드 입력**: 해상도(프리셋 또는 직접 입력), 주사율, 채널당 비트 심도(6/8/10/12), 크로마(RGB/4:4:4, 4:2:2, 4:2:0),
  타이밍 표준(CVT-R2 기본, CVT-RB, CVT 표준 블랭킹, H/V total 직접 입력).
- **결과**: 픽셀 클럭, 유효/전체 픽셀, 비압축 데이터율, 링크별 판정 표
  (HDMI 1.4 / 2.0 / 2.1 FRL 24·32·40·48G, DP 1.2 HBR2, DP 1.4 HBR3, DP 2.1 UHBR10/13.5/20):
  비압축 가능 / DSC로 가능(목표 bpp 선택, 압축비 표시) / 불가, 여유율(%).
- **최대 주사율 찾기**: 선택한 해상도·심도·링크에서 비압축/DSC 각각의 최대 주사율.
- **크기 계산**: 대각선 → 가로/세로(cm, in), PPI, 픽셀 피치, 1분각 기준 거리, 시청 거리에서의 PPD, 곡률(R)에 따른 곡면 깊이.
  두 모니터의 실제 비율 크기 비교(캔버스).
- 한국어/영어 전환(기본값은 브라우저 언어), 모바일 대응, 시스템 다크/라이트 모드,
  URL 프래그먼트(`#m=...`)로 상태 공유(서버로 전송되지 않음), 텍스트 요약 복사.

### 계산 방식과 가정
- **타이밍**: 공개된 VESA CVT 공식(프로그레시브, 마진 없음)을 직접 구현했습니다.
  - CVT-RB: H 블랭크 160 px, 최소 V 블랭크 460 µs, 0.25 MHz 클럭 단위.
  - CVT-R2: H 블랭크 80 px, V sync 8라인, 0.001 MHz 정밀도.
  - CVT: 최소 V sync+백포치 550 µs, 듀티 사이클 공식 `C' − M'·H주기`(C'=30, M'=300), 0.25 MHz 단위.
  - CVT v1의 V sync 폭은 화면비 표(4:3, 16:9, 16:10, 5:4, 15:9)로 정하고, 그 외(21:9, 32:9 …)는 10라인.
- **데이터율** = 픽셀 클럭 × 픽셀당 비트(심도 × 3 / × 2 / × 1.5). 블랭킹 구간 포함, 오디오 등 패킷 무시.
- **링크 실효 대역폭** = 원시 속도 × 라인 코딩 효율(8b/10b, 16b/18b, 128b/132b). FRL RS-FEC 등은 미반영.
- **DSC**: HDMI 2.1 FRL, DP 1.4, DP 2.1에서만 가정. 압축 데이터율 = 픽셀 클럭 × 목표 bpp (기본 RGB 12 bpp, 4:2:2 8, 4:2:0 6).
  DP 1.4에서 DSC 사용 시 FEC 오버헤드 약 2.4 % 적용.
- **크기**: 정사각형 픽셀 가정. 1분각 거리 = 픽셀 피치 / tan(1′). PPD = 2·d·tan(0.5°)·PPI.
- 모든 수치는 **대략적인 추정치**입니다. 실제 기기는 자체 EDID 타이밍·DSC 지원 범위가 다릅니다.

### 사용법
GitHub Pages 배포본을 열거나 로컬에서 빌드하세요. 입력을 바꾸면 결과와 URL `#m=...`가 즉시 갱신되므로 링크를 그대로 공유할 수 있습니다.

### 로컬 빌드
```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.22.1 --locked   # 또는 릴리스 바이너리
cargo test -p bw_core
trunk build --release        # 결과물: dist/ (public_url = /display-bw/)
sh tools/privacy_check.sh    # "clean" 이 출력되어야 함
```
개발 서버: `trunk serve --public-url /` 후 브라우저에서 열기.

### 구조
- `crates/bw_core` — 순수 Rust(웹 의존성 없음): 타이밍, 데이터율, 링크 표, DSC 판정, 최대 주사율 탐색, 크기 함수 + 단위 테스트.
- `crates/bw_web` — `web-sys` 기반 UI (프레임워크 없음).
- `index.html`, `web/` — 정적 HTML/CSS, 인라인 스크립트 없는 부트스트랩(`web/boot.js`).

### 라이선스
MIT — `LICENSE` 참고.

---

## English

A **display link bandwidth & monitor geometry calculator** for PC gamers and anyone buying high-refresh, ultrawide or
super-ultrawide monitors who asks "will this cable/port run 5120x1440 @ 240 Hz 10-bit, and do I need DSC or 4:2:2?".

It is a static single-page app written in Rust and compiled to WebAssembly. **Everything runs in your browser**:
no analytics, no trackers, no external CDNs or fonts, no network requests at runtime (enforced by the CSP).

### Features
- **Mode input**: resolution (presets or custom), refresh rate, bit depth per channel (6/8/10/12),
  chroma (RGB/4:4:4, 4:2:2, 4:2:0), timing standard (CVT-R2 default, CVT-RB, standard CVT, or custom H/V totals).
- **Results**: pixel clock, active vs total pixels, uncompressed data rate and a link table
  (HDMI 1.4 / 2.0 / 2.1 FRL 24/32/40/48G, DP 1.2 HBR2, DP 1.4 HBR3, DP 2.1 UHBR10/13.5/20) showing
  fits uncompressed / fits with DSC (selectable target bpp, compression ratio) / no, with headroom %.
- **Max refresh finder**: highest refresh rate that fits a chosen link, uncompressed and with DSC.
- **Geometry**: diagonal → width/height (cm & in), PPI, dot pitch, 1-arcminute viewing distance, pixels per degree at
  your viewing distance, curvature (R) arc-depth estimate, and a to-scale size comparison of two monitors.
- Korean/English toggle (default follows the browser), mobile friendly, follows system dark/light mode,
  state shareable via the URL fragment (`#m=...`, never sent to a server), copy-as-text summary.

### How the math works (and its assumptions)
- **Timing**: an independent implementation of the public VESA CVT formulas (progressive, no margins).
  - CVT-RB: 160 px horizontal blank, ≥ 460 µs vertical blank, 0.25 MHz clock step.
  - CVT-R2: 80 px horizontal blank, 8-line V-sync, 0.001 MHz precision.
  - CVT: ≥ 550 µs V-sync + back porch, duty-cycle blanking `C' − M'·H_period` (C'=30, M'=300), 0.25 MHz step.
  - CVT v1 picks V-sync width from the aspect table (4:3, 16:9, 16:10, 5:4, 15:9); other aspects (21:9, 32:9, …) use 10 lines.
- **Data rate** = pixel clock × bits per pixel (depth × 3 / × 2 / × 1.5). Blanking counts; audio and other packets are ignored.
- **Link payload** = raw rate × line-coding efficiency (8b/10b, 16b/18b, 128b/132b). FRL RS-FEC etc. not modelled.
- **DSC** is assumed on HDMI 2.1 FRL, DP 1.4 and DP 2.1 only. Compressed rate = pixel clock × target bpp
  (default 12 bpp RGB, 8 for 4:2:2, 6 for 4:2:0). An extra ≈2.4 % FEC overhead is applied for DSC on DP 1.4.
- **Geometry** assumes square pixels. 1-arcmin distance = pixel pitch / tan(1′). PPD = 2·d·tan(0.5°)·PPI.
- All figures are **approximate**. Real devices use their own EDID timings and DSC capabilities.

### Usage
Open the GitHub Pages build or build locally. Results and the `#m=...` fragment update as you type, so the URL can be shared as-is.

### Local build
```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.22.1 --locked   # or use the release binary
cargo test -p bw_core
trunk build --release        # output in dist/ (public_url = /display-bw/)
sh tools/privacy_check.sh    # must print "clean"
```
Dev server: `trunk serve --public-url /` and open it in a browser.

### Layout
- `crates/bw_core` — pure Rust (no web deps): timings, data rates, link table, DSC fit, max-refresh solver, geometry + unit tests.
- `crates/bw_web` — UI on plain `web-sys` (no framework).
- `index.html`, `web/` — static HTML/CSS and the inline-script-free bootstrap (`web/boot.js`).

### License
MIT — see `LICENSE`.
