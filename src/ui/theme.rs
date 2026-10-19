//! Colors, fonts and the egui style.

use std::sync::Arc;

/// (regular UI font, bold UI font, CJK fallbacks)
type SystemFonts = (Option<Vec<u8>>, Option<Vec<u8>>, Vec<Vec<u8>>);

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    TextStyle, Theme,
};

pub const BG: Color32 = Color32::from_rgb(0x0e, 0x10, 0x16);
pub const SURFACE: Color32 = Color32::from_rgb(0x16, 0x19, 0x22);
pub const SURFACE2: Color32 = Color32::from_rgb(0x1e, 0x22, 0x2e);
pub const SURFACE3: Color32 = Color32::from_rgb(0x27, 0x2c, 0x3b);
pub const BORDER: Color32 = Color32::from_rgb(0x2a, 0x2f, 0x3e);
pub const TEXT: Color32 = Color32::from_rgb(0xe8, 0xea, 0xf2);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x98, 0x9e, 0xb3);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x5d, 0x64, 0x78);
pub const ACCENT: Color32 = Color32::from_rgb(0x8b, 0x5c, 0xf6);
pub const ACCENT_HI: Color32 = Color32::from_rgb(0xa7, 0x8b, 0xfa);
pub const CYAN: Color32 = Color32::from_rgb(0x22, 0xd3, 0xee);
pub const GOOD: Color32 = Color32::from_rgb(0x34, 0xd3, 0x99);
pub const WARN: Color32 = Color32::from_rgb(0xfb, 0xbf, 0x24);
pub const BAD: Color32 = Color32::from_rgb(0xf8, 0x71, 0x71);

pub const RADIUS: u8 = 12;

/// Colors for EQ points, cycled by index.
pub const POINT_COLORS: [Color32; 8] = [
    Color32::from_rgb(0xa7, 0x8b, 0xfa),
    Color32::from_rgb(0x22, 0xd3, 0xee),
    Color32::from_rgb(0x34, 0xd3, 0x99),
    Color32::from_rgb(0xfb, 0xbf, 0x24),
    Color32::from_rgb(0xf4, 0x72, 0xb6),
    Color32::from_rgb(0x60, 0xa5, 0xfa),
    Color32::from_rgb(0xfb, 0x92, 0x3c),
    Color32::from_rgb(0xa3, 0xe6, 0x35),
];

pub fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

pub fn font(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn bold_font(size: f32) -> FontId {
    FontId::new(size, bold())
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

fn read_first(paths: &[&str]) -> Option<Vec<u8>> {
    paths.iter().find_map(|p| std::fs::read(p).ok())
}

#[cfg(windows)]
fn system_fonts(lang: &str) -> SystemFonts {
    let fonts = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into()) + r"\Fonts\";
    let p = |n: &str| format!("{fonts}{n}");
    let regular = read_first(&[&p("segoeui.ttf")]);
    let bold = read_first(&[&p("seguisb.ttf"), &p("segoeuib.ttf")]);
    let ja = [p("YuGothM.ttc"), p("meiryo.ttc"), p("msgothic.ttc")];
    let zh_cn = [p("msyh.ttc"), p("simsun.ttc")];
    let zh_tw = [p("msjh.ttc"), p("mingliu.ttc")];
    let ko = [p("malgun.ttf"), p("gulim.ttc")];
    (regular, bold, cjk(lang, &ja, &zh_cn, &zh_tw, &ko))
}

#[cfg(target_os = "macos")]
fn system_fonts(lang: &str) -> SystemFonts {
    let s = |n: &str| n.to_string();
    let ja = [
        s("/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc"),
        s("/System/Library/Fonts/Hiragino Sans GB.ttc"),
    ];
    let zh_cn = [
        s("/System/Library/Fonts/PingFang.ttc"),
        s("/System/Library/Fonts/Hiragino Sans GB.ttc"),
    ];
    let zh_tw = [s("/System/Library/Fonts/PingFang.ttc")];
    let ko = [s("/System/Library/Fonts/AppleSDGothicNeo.ttc")];
    (None, None, cjk(lang, &ja, &zh_cn, &zh_tw, &ko))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn system_fonts(lang: &str) -> SystemFonts {
    let noto: Vec<String> = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/OTF/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let _ = lang;
    (
        None,
        None,
        read_first(&noto.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .into_iter()
            .collect(),
    )
}

/// The CJK font for `lang` first (so its glyph shapes win), then one fallback.
#[allow(dead_code)]
fn cjk(
    lang: &str,
    ja: &[String],
    zh_cn: &[String],
    zh_tw: &[String],
    ko: &[String],
) -> Vec<Vec<u8>> {
    let order: [&[String]; 4] = match lang {
        "zh-CN" => [zh_cn, ja, zh_tw, ko],
        "zh-TW" => [zh_tw, ja, zh_cn, ko],
        "ko" => [ko, ja, zh_cn, zh_tw],
        _ => [ja, zh_cn, zh_tw, ko],
    };
    let mut out = Vec::new();
    for group in order {
        if let Some(bytes) = read_first(&group.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
            out.push(bytes);
            // One CJK font covers kanji/kana/hanzi; each is 10–20 MB in memory.
            break;
        }
    }
    out
}

pub fn install_fonts(ctx: &egui::Context, lang: &str) {
    let mut fonts = FontDefinitions::default();
    let (regular, bold_face, cjk) = system_fonts(lang);

    if let Some(r) = regular {
        fonts
            .font_data
            .insert("ui".into(), Arc::new(FontData::from_owned(r)));
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "ui".into());
    }
    let mut bold_family = fonts.families[&FontFamily::Proportional].clone();
    if let Some(b) = bold_face {
        fonts
            .font_data
            .insert("ui-bold".into(), Arc::new(FontData::from_owned(b)));
        bold_family.insert(0, "ui-bold".into());
    }
    // For CJK UIs the CJK font goes first: mixing its metrics with Segoe UI's
    // on one line puts Latin words on a different baseline.
    let cjk_first = matches!(lang, "ja" | "zh-CN" | "zh-TW" | "ko");
    for (i, bytes) in cjk.into_iter().enumerate() {
        let name = format!("cjk{i}");
        fonts
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
        let prop = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
        if cjk_first && i == 0 {
            prop.insert(0, name.clone());
            bold_family.insert(0, name.clone());
        } else {
            prop.push(name.clone());
            bold_family.push(name.clone());
        }
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push(name);
    }
    fonts.families.insert(bold(), bold_family);
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    // Phosphor registers itself for Proportional only; icons also appear in bold text.
    if let Some(last) = fonts.families[&FontFamily::Proportional]
        .iter()
        .find(|n| n.contains("phosphor"))
        .cloned()
    {
        fonts.families.get_mut(&bold()).unwrap().push(last);
    }
    ctx.set_fonts(fonts);
}

pub fn apply_style(ctx: &egui::Context) {
    ctx.set_theme(Theme::Dark);
    ctx.style_mut_of(Theme::Dark, |style| {
        style.text_styles = [
            (TextStyle::Small, font(11.0)),
            (TextStyle::Body, font(13.5)),
            (TextStyle::Button, font(13.5)),
            (TextStyle::Heading, bold_font(17.0)),
            (TextStyle::Monospace, mono(12.5)),
        ]
        .into();
        let s = &mut style.spacing;
        s.item_spacing = egui::vec2(8.0, 8.0);
        s.button_padding = egui::vec2(10.0, 5.0);
        s.interact_size.y = 28.0;
        s.slider_width = 200.0;
        s.combo_height = 320.0;
        s.menu_margin = Margin::same(6);
        s.window_margin = Margin::same(18);
        s.scroll.floating = true;

        let v = &mut style.visuals;
        v.dark_mode = true;
        v.panel_fill = BG;
        v.window_fill = SURFACE;
        v.extreme_bg_color = Color32::from_rgb(0x0b, 0x0d, 0x12);
        v.faint_bg_color = SURFACE2;
        v.window_stroke = Stroke::new(1.0, BORDER);
        v.window_corner_radius = CornerRadius::same(RADIUS);
        v.menu_corner_radius = CornerRadius::same(10);
        v.window_shadow.color = Color32::from_black_alpha(120);
        v.popup_shadow.color = Color32::from_black_alpha(110);
        v.selection.bg_fill = ACCENT.linear_multiply(0.55);
        v.selection.stroke = Stroke::new(1.0, ACCENT_HI);
        v.hyperlink_color = ACCENT_HI;
        v.slider_trailing_fill = true;
        v.handle_shape = egui::style::HandleShape::Circle;

        let w = &mut v.widgets;
        let r = CornerRadius::same(8);
        w.noninteractive.bg_fill = SURFACE2;
        w.noninteractive.weak_bg_fill = SURFACE2;
        w.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
        w.noninteractive.corner_radius = r;
        w.inactive.bg_fill = SURFACE3;
        w.inactive.weak_bg_fill = SURFACE2;
        w.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
        w.inactive.corner_radius = r;
        w.hovered.bg_fill = Color32::from_rgb(0x33, 0x39, 0x4c);
        w.hovered.weak_bg_fill = SURFACE3;
        w.hovered.bg_stroke = Stroke::new(1.0, ACCENT.linear_multiply(0.7));
        w.hovered.fg_stroke = Stroke::new(1.5, TEXT);
        w.hovered.corner_radius = r;
        w.active.bg_fill = ACCENT;
        w.active.weak_bg_fill = ACCENT.linear_multiply(0.6);
        w.active.bg_stroke = Stroke::new(1.0, ACCENT_HI);
        w.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
        w.active.corner_radius = r;
        w.open = w.hovered;
    });
}
