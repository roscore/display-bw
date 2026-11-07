//! UI strings (English / Korean).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Ko,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ko => "ko",
        }
    }
    pub fn from_code(s: &str) -> Option<Lang> {
        let s = s.to_ascii_lowercase();
        if s.starts_with("ko") {
            Some(Lang::Ko)
        } else if s.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }
    pub fn other(self) -> Lang {
        match self {
            Lang::En => Lang::Ko,
            Lang::Ko => Lang::En,
        }
    }
}

/// Look up a string. Unknown keys return "?" so gaps are visible.
pub fn t(lang: Lang, key: &str) -> &'static str {
    let (en, ko) = match key {
        "title" => ("Display link bandwidth & monitor calculator", "디스플레이 대역폭 · 모니터 계산기"),
        "subtitle" => (
            "Will this cable/port run your mode? Do you need DSC or 4:2:2? Runs entirely in your browser.",
            "이 케이블/포트로 원하는 모드가 나올까? DSC나 4:2:2가 필요할까? 모든 계산은 브라우저 안에서만 실행됩니다.",
        ),
        "lang_toggle" => ("한국어", "English"),
        "sec_mode" => ("Mode", "모드"),
        "resolution" => ("Resolution", "해상도"),
        "preset" => ("Preset", "프리셋"),
        "custom" => ("Custom", "직접 입력"),
        "refresh" => ("Refresh rate (Hz)", "주사율 (Hz)"),
        "depth" => ("Bit depth / channel", "채널당 비트 심도"),
        "chroma" => ("Chroma", "크로마"),
        "timing" => ("Timing standard", "타이밍 표준"),
        "timing_r2" => ("CVT-R2 (reduced blanking v2)", "CVT-R2 (축소 블랭킹 v2)"),
        "timing_rb" => ("CVT-RB (reduced blanking v1)", "CVT-RB (축소 블랭킹 v1)"),
        "timing_cvt" => ("CVT (standard blanking)", "CVT (표준 블랭킹)"),
        "timing_custom" => ("Custom totals", "직접 입력 (Total)"),
        "h_total" => ("H total", "H total"),
        "v_total" => ("V total", "V total"),
        "dsc_bpp" => ("DSC target (bpp)", "DSC 목표 (bpp)"),
        "dsc_auto" => ("auto", "자동"),
        "sec_results" => ("Results", "결과"),
        "pixel_clock" => ("Pixel clock", "픽셀 클럭"),
        "totals" => ("Total (H × V)", "Total (H × V)"),
        "active_total" => ("Active / total pixels", "유효 / 전체 픽셀"),
        "blanking" => ("Blanking overhead", "블랭킹 오버헤드"),
        "actual_refresh" => ("Actual refresh", "실제 주사율"),
        "h_freq" => ("Horizontal frequency", "수평 주파수"),
        "bpp" => ("Bits per pixel", "픽셀당 비트"),
        "data_rate" => ("Uncompressed data rate", "비압축 데이터율"),
        "col_link" => ("Link", "링크"),
        "col_payload" => ("Payload", "실효 대역폭"),
        "col_uncompressed" => ("Uncompressed", "비압축"),
        "col_dsc" => ("With DSC", "DSC 사용"),
        "col_max" => ("Max Hz (raw / DSC)", "최대 Hz (비압축 / DSC)"),
        "fit_yes" => ("fits", "가능"),
        "fit_no" => ("no", "불가"),
        "fit_dsc" => ("DSC", "DSC 필요"),
        "dsc_unsupported" => ("not supported", "미지원"),
        "dsc_na" => ("n/a (target ≥ source bpp)", "해당 없음 (목표 ≥ 원본 bpp)"),
        "headroom" => ("headroom", "여유"),
        "ratio" => ("ratio", "압축비"),
        "approx" => (
            "All figures are approximate planning estimates, not a guarantee that a specific device pair works.",
            "모든 수치는 대략적인 추정치이며, 특정 기기 조합의 동작을 보장하지 않습니다.",
        ),
        "sec_max" => ("Max refresh finder", "최대 주사율 찾기"),
        "max_link" => ("Link", "링크"),
        "max_raw" => ("Highest refresh, uncompressed", "비압축 최대 주사율"),
        "max_dsc" => ("Highest refresh, with DSC", "DSC 사용 시 최대 주사율"),
        "none" => ("does not fit", "불가"),
        "sec_geo" => ("Geometry", "크기 · 밀도"),
        "monitor_a" => ("Monitor A (uses the mode resolution)", "모니터 A (위 모드 해상도 사용)"),
        "monitor_b" => ("Monitor B (comparison)", "모니터 B (비교용)"),
        "diagonal" => ("Diagonal (in)", "대각선 (인치)"),
        "distance" => ("Viewing distance (cm)", "시청 거리 (cm)"),
        "curve" => ("Curvature radius (R, mm; 0 = flat)", "곡률 반경 (R, mm; 0 = 평면)"),
        "aspect" => ("Aspect", "화면비"),
        "size" => ("Width × height", "가로 × 세로"),
        "ppi" => ("Pixel density", "픽셀 밀도"),
        "pitch" => ("Dot pitch", "픽셀 피치"),
        "retina" => ("1-arcmin distance", "1분각 거리"),
        "retina_hint" => ("pixels become indistinguishable beyond this", "이 거리보다 멀면 픽셀 구분 불가"),
        "ppd" => ("Pixels per degree", "도당 픽셀 (PPD)"),
        "fov" => ("Horizontal field of view", "수평 시야각"),
        "curve_depth" => ("Arc depth (sagitta)", "곡면 깊이 (사지타)"),
        "curve_chord" => ("Edge-to-edge (chord)", "양끝 직선 거리 (현)"),
        "curve_angle" => ("Arc angle", "호 각도"),
        "flat" => ("flat", "평면"),
        "compare" => ("Size comparison (to scale)", "크기 비교 (실제 비율)"),
        "area_ratio" => ("Area B / A", "면적 B / A"),
        "copy" => ("Copy summary", "결과 복사"),
        "share" => ("Copy share link", "공유 링크 복사"),
        "copied" => ("Copied.", "복사했습니다."),
        "copy_failed" => ("Copy blocked — select the text below manually.", "복사가 차단됨 — 아래 텍스트를 직접 선택하세요."),
        "summary" => ("Plain-text summary", "텍스트 요약"),
        "err" => ("Invalid input", "입력 오류"),
        "privacy" => (
            "No tracking, no network requests. Settings live only in the URL fragment (#m=…) and, for the language, in your browser.",
            "추적·네트워크 요청 없음. 설정은 URL 프래그먼트(#m=…)와 브라우저(언어 설정)에만 저장됩니다.",
        ),
        "notes_title" => ("Assumptions & simplifications", "가정 및 단순화"),
        "notes_html" => (NOTES_EN, NOTES_KO),
        _ => ("?", "?"),
    };
    match lang {
        Lang::En => en,
        Lang::Ko => ko,
    }
}

const NOTES_EN: &str = "<ul>\
<li><b>Timing.</b> CVT, CVT-RB and CVT-R2 follow the public VESA CVT formulas (progressive, no margins). \
CVT v1 picks the V-sync width from the aspect ratio; aspects outside its table (21:9, 32:9, …) use 10 lines. \
Real monitors often ship their own EDID timings, which can be tighter or looser — use <i>Custom totals</i> to enter them.</li>\
<li><b>Data rate</b> = pixel clock × bits per pixel, so blanking time counts against the link. \
Bits per pixel = depth × 3 (RGB/4:4:4), × 2 (4:2:2) or × 1.5 (4:2:0). Audio, InfoFrames and other packets are ignored.</li>\
<li><b>Payload</b> = raw link rate × line-coding efficiency: 8b/10b (HDMI TMDS, DP 1.x) = 80 %, \
16b/18b (HDMI 2.1 FRL) ≈ 88.9 %, 128b/132b (DP 2.x UHBR) ≈ 97 %. FRL Reed-Solomon FEC, DP 2.x FEC and \
micro-packet overheads are not modelled, so the top of each link is slightly optimistic.</li>\
<li><b>DSC</b> is assumed on HDMI 2.1 FRL, DP 1.4 and DP 2.1 (not on TMDS HDMI or DP 1.2). Compressed rate = \
pixel clock × target bpp. On DP 1.4 an extra ≈2.4 % FEC overhead is applied when DSC is used. \
Whether a GPU/monitor pair supports a given DSC target, slice count or pixel rate is device-specific.</li>\
<li><b>HDMI TMDS 4:2:2</b> is actually sent in a fixed 24-bit container; this tool uses the generic \
depth-based formula for every link.</li>\
<li><b>Thunderbolt / USB4:</b> 40 Gbit/s ports usually tunnel or alt-mode a DP 1.4 HBR3 ×4 stream — use the DP 1.4 row. \
Newer 80 Gbit/s generations may offer UHBR rates; check the device.</li>\
<li><b>Geometry</b> assumes square pixels. Curvature treats the stated width as the arc length.</li>\
</ul>";

const NOTES_KO: &str = "<ul>\
<li><b>타이밍.</b> CVT, CVT-RB, CVT-R2는 공개된 VESA CVT 공식(프로그레시브, 마진 없음)을 따릅니다. \
CVT v1은 화면비로 V-sync 폭을 정하며, 표에 없는 화면비(21:9, 32:9 등)는 10라인을 사용합니다. \
실제 모니터는 자체 EDID 타이밍을 쓰는 경우가 많으니, 필요하면 <i>직접 입력 (Total)</i>을 사용하세요.</li>\
<li><b>데이터율</b> = 픽셀 클럭 × 픽셀당 비트. 블랭킹 구간도 링크 시간을 차지하므로 포함됩니다. \
픽셀당 비트 = 심도 × 3 (RGB/4:4:4), × 2 (4:2:2), × 1.5 (4:2:0). 오디오·InfoFrame 등 패킷은 무시합니다.</li>\
<li><b>실효 대역폭</b> = 링크 원시 속도 × 라인 코딩 효율: 8b/10b (HDMI TMDS, DP 1.x) = 80 %, \
16b/18b (HDMI 2.1 FRL) ≈ 88.9 %, 128b/132b (DP 2.x UHBR) ≈ 97 %. FRL RS-FEC, DP 2.x FEC, \
마이크로 패킷 오버헤드는 반영하지 않아 한계 근처는 약간 낙관적입니다.</li>\
<li><b>DSC</b>는 HDMI 2.1 FRL, DP 1.4, DP 2.1에서만 가능하다고 가정합니다 (TMDS HDMI, DP 1.2는 불가). \
압축 데이터율 = 픽셀 클럭 × 목표 bpp. DP 1.4에서 DSC 사용 시 FEC 오버헤드 약 2.4 %를 추가로 적용합니다. \
실제 GPU/모니터의 DSC 목표값·슬라이스 수·픽셀 레이트 지원 여부는 기기마다 다릅니다.</li>\
<li><b>HDMI TMDS 4:2:2</b>는 실제로 고정 24비트 컨테이너로 전송되지만, 이 도구는 모든 링크에 \
심도 기반의 일반 공식을 사용합니다.</li>\
<li><b>Thunderbolt / USB4:</b> 40 Gbit/s 포트는 보통 DP 1.4 HBR3 ×4 스트림을 터널링/Alt 모드로 전달합니다 — DP 1.4 행을 참고하세요. \
80 Gbit/s 세대는 UHBR 속도를 지원할 수 있으니 기기 사양을 확인하세요.</li>\
<li><b>크기 계산</b>은 정사각형 픽셀을 가정합니다. 곡률 계산은 표기된 가로 길이를 호의 길이로 봅니다.</li>\
</ul>";
