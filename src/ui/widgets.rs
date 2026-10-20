//! Small custom widgets.

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, Frame, Margin, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, WidgetText,
};

use super::theme::*;

pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// A card padded to at least `min_height` of content; also returns the height
/// its content needed, so cards side by side can be given the same height.
pub fn card_min_height<R>(
    ui: &mut Ui,
    min_height: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> (R, f32) {
    card(ui, |ui| {
        let top = ui.min_rect().top();
        let r = add(ui);
        let needed = ui.min_rect().bottom() - top;
        // Pad only the difference. (`set_min_height` would reserve the whole
        // height again below the content, from the cursor.)
        let pad = min_height - needed - ui.spacing().item_spacing.y;
        if pad > 0.0 {
            ui.add_space(pad);
        }
        (r, needed)
    })
}

pub fn section_title(ui: &mut Ui, icon: &str, text: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).font(font(16.0)).color(ACCENT_HI));
        ui.label(egui::RichText::new(text).font(bold_font(14.5)).color(TEXT));
    });
}

pub fn dim(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text.into())
        .color(TEXT_DIM)
        .font(font(12.0))
}

pub fn faint(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text.into())
        .color(TEXT_FAINT)
        .font(font(11.5))
}

/// iOS-style switch.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    toggle_sized(ui, on, 1.0)
}

pub fn toggle_sized(ui: &mut Ui, on: &mut bool, scale: f32) -> Response {
    let size = vec2(40.0, 22.0) * scale;
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool_responsive(resp.id, *on);
    if ui.is_rect_visible(rect) {
        let off_fill = if resp.hovered() {
            SURFACE3.gamma_multiply(1.3)
        } else {
            SURFACE3
        };
        let fill = lerp_color(off_fill, ACCENT, t);
        let r = rect.height() / 2.0;
        ui.painter().rect_filled(rect, r, fill);
        let x = egui::lerp(rect.left() + r..=rect.right() - r, t);
        ui.painter()
            .circle_filled(pos2(x, rect.center().y), r - 3.0 * scale, Color32::WHITE);
    }
    resp
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

/// Segmented control; returns the newly picked index. `tips` (if any) are
/// hover texts, one per label.
pub fn segmented(
    ui: &mut Ui,
    labels: &[String],
    tips: &[String],
    selected: usize,
) -> Option<usize> {
    let mut picked = None;
    let font_id = font(12.5);
    let widths: Vec<f32> = labels
        .iter()
        .map(|l| {
            ui.painter()
                .layout_no_wrap(l.clone(), font_id.clone(), TEXT)
                .size()
                .x
                + 24.0
        })
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 6.0;
    let (rect, _) = ui.allocate_exact_size(vec2(total, 30.0), Sense::hover());
    ui.painter().rect_filled(rect, 9.0, SURFACE2);
    ui.painter()
        .rect_stroke(rect, 9.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
    let mut x = rect.left() + 3.0;
    for (i, (label, w)) in labels.iter().zip(widths).enumerate() {
        let r = Rect::from_min_size(pos2(x, rect.top() + 3.0), vec2(w, rect.height() - 6.0));
        let mut resp = ui.interact(r, ui.id().with(("seg", i, label)), Sense::click());
        if let Some(tip) = tips.get(i) {
            resp = resp.on_hover_text(tip);
        }
        if i == selected {
            ui.painter().rect_filled(r, 7.0, ACCENT);
        } else if resp.hovered() {
            ui.painter().rect_filled(r, 7.0, SURFACE3);
        }
        let color = if i == selected {
            Color32::WHITE
        } else {
            TEXT_DIM
        };
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            label,
            font_id.clone(),
            color,
        );
        if resp.clicked() && i != selected {
            picked = Some(i);
        }
        x += w;
    }
    picked
}

/// Horizontal level meter. `level` and `marker` are 0..1.
pub fn meter(ui: &mut Ui, level: f32, peak_hold: f32, height: f32, clip: bool) -> Rect {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    paint_meter(ui, rect, level, peak_hold, clip);
    rect
}

pub fn paint_meter(ui: &Ui, rect: Rect, level: f32, peak_hold: f32, clip: bool) {
    let p = ui.painter();
    let r = rect.height() / 2.0;
    p.rect_filled(rect, r, Color32::from_rgb(0x0b, 0x0d, 0x12));
    let level = level.clamp(0.0, 1.0);
    if level > 0.002 {
        let fill = Rect::from_min_max(
            rect.min,
            pos2(rect.left() + rect.width() * level, rect.bottom()),
        );
        // Green → amber → red as the level rises.
        let color = if level > 0.92 || clip {
            BAD
        } else if level > 0.75 {
            lerp_color(GOOD, WARN, (level - 0.75) / 0.17)
        } else {
            lerp_color(CYAN, GOOD, level / 0.75)
        };
        p.rect_filled(fill, r, color);
    }
    if peak_hold > 0.01 {
        let x = rect.left() + rect.width() * peak_hold.clamp(0.0, 1.0);
        p.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(2.0, TEXT),
        );
    }
}

pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    let resp = ui.add(
        egui::Button::new(egui::RichText::new(icon).font(font(16.0)))
            .frame(false)
            .min_size(vec2(30.0, 30.0)),
    );
    resp.on_hover_text(tooltip)
}

pub fn primary_button(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.add(
        egui::Button::new(text.into().color(Color32::WHITE))
            .fill(ACCENT)
            .min_size(vec2(0.0, 30.0)),
    )
}

pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font(11.0), color);
    let size = galley.size() + vec2(12.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, rect.height() / 2.0, color.linear_multiply(0.18));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, color);
}

/// dBFS → 0..1 for meters (−60 dB … 0 dB).
pub fn db_to_meter(db: f32) -> f32 {
    ((db + 60.0) / 60.0).clamp(0.0, 1.0)
}

pub fn lin_to_meter(lin: f32) -> f32 {
    db_to_meter(20.0 * lin.max(1e-6).log10())
}
