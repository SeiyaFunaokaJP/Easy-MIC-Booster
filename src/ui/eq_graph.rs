//! Frequency-response graph: live spectrum, the real filter response (not a
//! point-to-point polyline like v1.x), draggable EQ points and the gain line.

use eframe::egui::{
    self, epaint::Mesh, pos2, vec2, Align2, Color32, Key, Pos2, Rect, Sense, Shape, Stroke,
    StrokeKind, Ui,
};

use super::theme::*;
use super::widgets::lerp_color;
use crate::config::{BandType, EqBand, EqMode, Sound, MAX_EQ_POINTS};
use crate::dsp::biquad::eq_response_db;
use crate::dsp::{FFT_SIZE, SAMPLE_RATE};
use crate::i18n::tr;

const F_MIN: f32 = 20.0;
const F_MAX: f32 = 20_000.0;
const BAND_GAIN_LIMIT: f32 = 24.0;

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Point(usize),
    Gain,
}

pub struct EqGraph {
    pub selected: Option<usize>,
    drag: Option<Drag>,
    range: (f32, f32),
    menu_target: Option<usize>,
    menu_pos: Pos2,
    columns: Vec<f32>,
}

impl Default for EqGraph {
    fn default() -> Self {
        EqGraph {
            selected: None,
            drag: None,
            range: (-18.0, 36.0),
            menu_target: None,
            menu_pos: Pos2::ZERO,
            columns: Vec::new(),
        }
    }
}

struct Axes {
    plot: Rect,
    lo: f32,
    hi: f32,
}

impl Axes {
    fn x(&self, f: f32) -> f32 {
        let t = (f.clamp(F_MIN, F_MAX) / F_MIN).log10() / (F_MAX / F_MIN).log10();
        self.plot.left() + t * self.plot.width()
    }
    fn f(&self, x: f32) -> f32 {
        let t = ((x - self.plot.left()) / self.plot.width()).clamp(0.0, 1.0);
        F_MIN * (F_MAX / F_MIN).powf(t)
    }
    fn y(&self, db: f32) -> f32 {
        let t = (self.hi - db) / (self.hi - self.lo);
        self.plot.top() + t * self.plot.height()
    }
    fn db(&self, y: f32) -> f32 {
        self.hi - (y - self.plot.top()) / self.plot.height() * (self.hi - self.lo)
    }
}

pub fn format_freq(f: f32) -> String {
    if f >= 1000.0 {
        let k = f / 1000.0;
        if k >= 10.0 {
            format!("{k:.1} kHz")
        } else {
            format!("{k:.2} kHz")
        }
    } else {
        format!("{f:.0} Hz")
    }
}

pub fn band_type_label(t: BandType) -> String {
    tr(match t {
        BandType::Peaking => "Band_Peaking",
        BandType::LowShelf => "Band_LowShelf",
        BandType::HighShelf => "Band_HighShelf",
        BandType::HighPass => "Band_HighPass",
        BandType::LowPass => "Band_LowPass",
    })
}

impl EqGraph {
    fn point_pos(ax: &Axes, sound: &Sound, b: &EqBand) -> Pos2 {
        let db = if b.band_type.has_gain() {
            sound.gain_db + b.gain
        } else {
            sound.gain_db
        };
        pos2(ax.x(b.frequency), ax.y(db))
    }

    fn update_range(&mut self, sound: &Sound) {
        if self.drag.is_some() {
            return;
        }
        let bands = sound.active_bands();
        let max_b = bands
            .iter()
            .filter(|b| b.band_type.has_gain())
            .map(|b| b.gain)
            .fold(0.0f32, f32::max);
        let min_b = bands
            .iter()
            .filter(|b| b.band_type.has_gain())
            .map(|b| b.gain)
            .fold(0.0f32, f32::min);
        let hi = (sound.gain_db + max_b).max(0.0) + 8.0;
        let lo = (sound.gain_db + min_b).min(0.0) - 8.0;
        let hi = ((hi / 6.0).ceil() * 6.0).max(18.0);
        let lo = ((lo / 6.0).floor() * 6.0).min(-18.0);
        self.range = (lo, hi);
    }

    /// Draws the graph; returns the edited sound if the user changed anything.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        sound: &Sound,
        boost: bool,
        max_gain: f32,
        spectrum: Option<&[f32]>,
        height: f32,
    ) -> Option<Sound> {
        self.update_range(sound);
        let (rect, resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click_and_drag());
        let ax = Axes {
            plot: Rect::from_min_max(rect.min + vec2(40.0, 10.0), rect.max - vec2(14.0, 22.0)),
            lo: self.range.0,
            hi: self.range.1,
        };
        let painter = ui.painter_at(rect);
        let eq_mode = sound.mode == EqMode::Equalizer;
        let mut out: Option<Sound> = None;

        painter.rect_filled(
            rect,
            RADIUS as f32 - 2.0,
            Color32::from_rgb(0x0b, 0x0d, 0x12),
        );

        // Grid.
        for f in [
            20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10_000.0, 20_000.0,
        ] {
            let x = ax.x(f);
            painter.line_segment(
                [pos2(x, ax.plot.top()), pos2(x, ax.plot.bottom())],
                Stroke::new(1.0, BORDER.linear_multiply(0.6)),
            );
            let label = if f >= 1000.0 {
                format!("{}k", f / 1000.0)
            } else {
                format!("{f}")
            };
            painter.text(
                pos2(x, ax.plot.bottom() + 4.0),
                Align2::CENTER_TOP,
                label,
                font(10.5),
                TEXT_FAINT,
            );
        }
        let step = if ax.hi - ax.lo > 60.0 { 12.0 } else { 6.0 };
        let mut db = (ax.lo / step).ceil() * step;
        while db <= ax.hi {
            let y = ax.y(db);
            let zero = db.abs() < 0.01;
            let stroke = if zero {
                Stroke::new(1.0, TEXT_FAINT)
            } else {
                Stroke::new(1.0, BORDER.linear_multiply(0.6))
            };
            painter.line_segment([pos2(ax.plot.left(), y), pos2(ax.plot.right(), y)], stroke);
            painter.text(
                pos2(ax.plot.left() - 6.0, y),
                Align2::RIGHT_CENTER,
                format!("{db:+.0}"),
                font(10.5),
                TEXT_FAINT,
            );
            db += step;
        }

        // Live spectrum (own axis: −90…0 dBFS over the plot height).
        if let Some(spec) = spectrum {
            self.paint_spectrum(ui, &ax, spec);
        }

        // Response curve.
        let bands = sound.active_bands();
        let n = 220;
        let curve: Vec<Pos2> = (0..=n)
            .map(|i| {
                let f = F_MIN * (F_MAX / F_MIN).powf(i as f32 / n as f32);
                let db = sound.gain_db + eq_response_db(bands, f as f64, SAMPLE_RATE as f64) as f32;
                pos2(ax.x(f), ax.y(db.clamp(ax.lo - 40.0, ax.hi + 40.0)))
            })
            .collect();
        let color = if boost { ACCENT_HI } else { TEXT_FAINT };
        let zero_y = ax.y(0.0).clamp(ax.plot.top(), ax.plot.bottom());
        let mut mesh = Mesh::default();
        for (i, p) in curve.iter().enumerate() {
            let py = p.y.clamp(ax.plot.top(), ax.plot.bottom());
            mesh.colored_vertex(
                pos2(p.x, py),
                color.linear_multiply(if boost { 0.22 } else { 0.08 }),
            );
            mesh.colored_vertex(pos2(p.x, zero_y), color.linear_multiply(0.02));
            if i > 0 {
                let k = (i * 2) as u32;
                mesh.add_triangle(k - 2, k - 1, k);
                mesh.add_triangle(k - 1, k, k + 1);
            }
        }
        painter.add(Shape::mesh(mesh));
        let clip = ax.plot.expand(2.0);
        ui.painter_at(clip)
            .add(Shape::line(curve, Stroke::new(2.5, color)));

        // Gain handle on the left edge.
        let gain_y = ax.y(sound.gain_db).clamp(ax.plot.top(), ax.plot.bottom());
        let handle = Rect::from_center_size(pos2(ax.plot.left() + 22.0, gain_y), vec2(40.0, 18.0));
        let pointer = resp
            .hover_pos()
            .or_else(|| ui.input(|i| i.pointer.interact_pos()));
        let over_handle = pointer.is_some_and(|p| handle.expand(4.0).contains(p));
        let handle_active = over_handle || self.drag == Some(Drag::Gain);
        painter.rect_filled(
            handle,
            9.0,
            if handle_active {
                ACCENT
            } else {
                ACCENT.linear_multiply(0.75)
            },
        );
        painter.text(
            handle.center(),
            Align2::CENTER_CENTER,
            "GAIN",
            bold_font(10.0),
            Color32::WHITE,
        );
        painter.text(
            pos2(ax.plot.right() - 6.0, gain_y - 4.0),
            Align2::RIGHT_BOTTOM,
            crate::tray::format_db(sound.gain_db),
            bold_font(12.0),
            color,
        );

        // Points.
        let mut hovered: Option<usize> = None;
        if eq_mode {
            let mut best = 14.0f32;
            if let Some(p) = pointer {
                for (i, b) in bands.iter().enumerate() {
                    let d = Self::point_pos(&ax, sound, b).distance(p);
                    if d < best {
                        best = d;
                        hovered = Some(i);
                    }
                }
            }
            for (i, b) in bands.iter().enumerate() {
                let pos = Self::point_pos(&ax, sound, b);
                let pc = POINT_COLORS[i % POINT_COLORS.len()];
                let active = self.selected == Some(i) || self.drag == Some(Drag::Point(i));
                let r = if active || hovered == Some(i) {
                    9.0
                } else {
                    7.5
                };
                if active {
                    painter.circle_filled(pos, r + 5.0, pc.linear_multiply(0.18));
                }
                painter.circle_filled(
                    pos,
                    r,
                    if boost {
                        pc
                    } else {
                        lerp_color(pc, TEXT_FAINT, 0.6)
                    },
                );
                painter.circle_stroke(
                    pos,
                    r,
                    Stroke::new(1.5, Color32::from_rgb(0x0b, 0x0d, 0x12)),
                );
                painter.text(
                    pos,
                    Align2::CENTER_CENTER,
                    format!("{}", i + 1),
                    bold_font(10.0),
                    Color32::from_rgb(0x0b, 0x0d, 0x12),
                );
            }
            if let Some(i) = hovered.or(match self.drag {
                Some(Drag::Point(i)) => Some(i),
                _ => None,
            }) {
                if let Some(b) = bands.get(i) {
                    let mut text = format!(
                        "{} · {}",
                        band_type_label(b.band_type),
                        format_freq(b.frequency)
                    );
                    if b.band_type.has_gain() {
                        text += &format!(" · {:+.1} dB", b.gain);
                    }
                    if b.band_type.has_q() {
                        text += &format!(" · Q {:.2}", b.q);
                    }
                    let pos = Self::point_pos(&ax, sound, b);
                    tooltip(&painter, rect, pos, &text);
                }
            }
        } else if resp.hovered() && self.drag.is_none() {
            painter.text(
                pos2(ax.plot.center().x, ax.plot.top() + 6.0),
                Align2::CENTER_TOP,
                tr("Graph_SimpleHint"),
                font(11.5),
                TEXT_DIM,
            );
        }
        if !boost {
            painter.text(
                ax.plot.center(),
                Align2::CENTER_CENTER,
                tr("Graph_Bypassed"),
                bold_font(15.0),
                TEXT_FAINT,
            );
        }

        // ---- interaction ----
        let mut edited = sound.clone();
        let mut changed = false;

        if resp.drag_started() {
            self.drag = if over_handle || !eq_mode {
                Some(Drag::Gain)
            } else if let Some(i) = hovered {
                self.selected = Some(i);
                Some(Drag::Point(i))
            } else {
                None
            };
        }
        if resp.dragged() {
            if let Some(p) = resp.interact_pointer_pos() {
                match self.drag {
                    Some(Drag::Gain) => {
                        edited.gain_db = (ax.db(p.y) * 10.0).round() / 10.0;
                        edited.gain_db = edited.gain_db.clamp(-30.0, max_gain);
                        changed = true;
                    }
                    Some(Drag::Point(i)) if i < edited.bands.len() => {
                        let b = &mut edited.bands[i];
                        b.frequency = round_freq(ax.f(p.x));
                        if b.band_type.has_gain() {
                            b.gain = ((ax.db(p.y) - sound.gain_db) * 10.0).round() / 10.0;
                            b.gain = b.gain.clamp(-BAND_GAIN_LIMIT, BAND_GAIN_LIMIT);
                        }
                        changed = true;
                    }
                    _ => {}
                }
            }
        }
        if resp.drag_stopped() {
            self.drag = None;
        }
        if eq_mode && resp.clicked() {
            self.selected = hovered;
        }
        if eq_mode && resp.double_clicked() {
            match hovered {
                Some(i) => {
                    if edited.bands[i].band_type.has_gain() {
                        edited.bands[i].gain = 0.0;
                        changed = true;
                    }
                }
                None => {
                    if let Some(p) = resp.interact_pointer_pos() {
                        if edited.bands.len() < MAX_EQ_POINTS {
                            let g = ((ax.db(p.y) - sound.gain_db) * 10.0).round() / 10.0;
                            edited.bands.push(EqBand::peaking(
                                round_freq(ax.f(p.x)),
                                g.clamp(-BAND_GAIN_LIMIT, BAND_GAIN_LIMIT),
                                1.0,
                            ));
                            self.selected = Some(edited.bands.len() - 1);
                            changed = true;
                        }
                    }
                }
            }
        }
        // Mouse wheel over a point changes its width.
        if let Some(i) = hovered {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 && edited.bands[i].band_type.has_q() {
                let q = edited.bands[i].q * (1.0 + scroll * 0.004);
                edited.bands[i].q = (q.clamp(0.1, 12.0) * 100.0).round() / 100.0;
                changed = true;
            }
        }
        if eq_mode
            && resp.hovered()
            && ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace))
        {
            if let Some(i) = self.selected.filter(|&i| i < edited.bands.len()) {
                edited.bands.remove(i);
                self.selected = None;
                changed = true;
            }
        }

        if eq_mode {
            if resp.secondary_clicked() {
                self.menu_target = hovered;
                self.menu_pos = resp.interact_pointer_pos().unwrap_or(rect.center());
            }
            resp.context_menu(|ui| {
                ui.set_min_width(170.0);
                match self.menu_target.filter(|&i| i < edited.bands.len()) {
                    Some(i) => {
                        ui.label(faint_label(&format!(
                            "#{}  {}",
                            i + 1,
                            format_freq(edited.bands[i].frequency)
                        )));
                        for t in BandType::ALL {
                            if ui
                                .radio(edited.bands[i].band_type == t, band_type_label(t))
                                .clicked()
                            {
                                set_band_type(&mut edited.bands[i], t);
                                changed = true;
                                ui.close();
                            }
                        }
                        ui.separator();
                        if ui
                            .button(format!(
                                "{}  {}",
                                egui_phosphor::regular::TRASH,
                                tr("Eq_DeletePoint")
                            ))
                            .clicked()
                        {
                            edited.bands.remove(i);
                            self.selected = None;
                            changed = true;
                            ui.close();
                        }
                    }
                    None => {
                        let can_add = edited.bands.len() < MAX_EQ_POINTS;
                        if ui
                            .add_enabled(
                                can_add,
                                egui::Button::new(format!(
                                    "{}  {}",
                                    egui_phosphor::regular::PLUS,
                                    tr("Eq_AddPointHere")
                                )),
                            )
                            .clicked()
                        {
                            let p = self.menu_pos;
                            let g = ((ax.db(p.y) - sound.gain_db) * 10.0).round() / 10.0;
                            edited.bands.push(EqBand::peaking(
                                round_freq(ax.f(p.x)),
                                g.clamp(-BAND_GAIN_LIMIT, BAND_GAIN_LIMIT),
                                1.0,
                            ));
                            self.selected = Some(edited.bands.len() - 1);
                            changed = true;
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                !edited.bands.is_empty(),
                                egui::Button::new(format!(
                                    "{}  {}",
                                    egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE,
                                    tr("Eq_Reset")
                                )),
                            )
                            .clicked()
                        {
                            edited.bands.clear();
                            self.selected = None;
                            changed = true;
                            ui.close();
                        }
                    }
                }
            });
        }

        if over_handle || hovered.is_some() || self.drag.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        } else if !eq_mode && resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }

        if self.selected.is_some_and(|i| i >= edited.bands.len()) {
            self.selected = None;
        }
        if changed && edited != *sound {
            out = Some(edited);
        }
        out
    }

    /// Whether the spectrum still shows (or is fading out) a signal.
    pub fn spectrum_active(&self) -> bool {
        self.columns.iter().any(|&v| v > -88.0)
    }

    fn paint_spectrum(&mut self, ui: &Ui, ax: &Axes, spec: &[f32]) {
        let col_w = 3.0;
        let cols = (ax.plot.width() / col_w) as usize + 1;
        if self.columns.len() != cols {
            self.columns = vec![-120.0; cols];
        }
        let bin_hz = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let to_y = |db: f32| {
            let t = ((db + 90.0) / 90.0).clamp(0.0, 1.0);
            ax.plot.bottom() - t * ax.plot.height()
        };
        let mut mesh = Mesh::default();
        for c in 0..cols {
            let x0 = ax.plot.left() + c as f32 * col_w;
            let f0 = ax.f(x0);
            let f1 = ax.f(x0 + col_w);
            let b0 = ((f0 / bin_hz) as usize).clamp(1, spec.len() - 1);
            let b1 = ((f1 / bin_hz).ceil() as usize).clamp(b0 + 1, spec.len());
            let v = spec[b0..b1].iter().copied().fold(-120.0f32, f32::max);
            // Fast rise, slow fall.
            let d = &mut self.columns[c];
            *d = if v > *d { v } else { *d - (*d - v) * 0.18 };
            let y = to_y(*d);
            let top = lerp_color(
                CYAN.linear_multiply(0.35),
                ACCENT.linear_multiply(0.35),
                c as f32 / cols as f32,
            );
            mesh.colored_vertex(pos2(x0, y), top);
            mesh.colored_vertex(pos2(x0, ax.plot.bottom()), top.linear_multiply(0.05));
            if c > 0 {
                let k = (c * 2) as u32;
                mesh.add_triangle(k - 2, k - 1, k);
                mesh.add_triangle(k - 1, k, k + 1);
            }
        }
        ui.painter_at(ax.plot).add(Shape::mesh(mesh));
    }
}

fn faint_label(s: &str) -> egui::RichText {
    egui::RichText::new(s).color(TEXT_FAINT).font(font(11.0))
}

pub fn set_band_type(b: &mut EqBand, t: BandType) {
    b.band_type = t;
    if !t.has_gain() {
        b.gain = 0.0;
        if b.q < 0.3 {
            b.q = 0.71;
        }
    }
    if t.has_q() && b.q < 0.1 {
        b.q = 1.0;
    }
}

fn round_freq(f: f32) -> f32 {
    let f = f.clamp(F_MIN, F_MAX);
    if f < 100.0 {
        f.round()
    } else if f < 1000.0 {
        (f / 5.0).round() * 5.0
    } else {
        (f / 50.0).round() * 50.0
    }
}

fn tooltip(painter: &egui::Painter, bounds: Rect, anchor: Pos2, text: &str) {
    let galley = painter.layout_no_wrap(text.to_string(), font(11.5), TEXT);
    let size = galley.size() + vec2(14.0, 8.0);
    let mut r = Rect::from_min_size(anchor + vec2(12.0, -size.y - 10.0), size);
    if r.right() > bounds.right() - 4.0 {
        r = r.translate(vec2(-(size.x + 24.0), 0.0));
    }
    if r.top() < bounds.top() + 4.0 {
        r = r.translate(vec2(0.0, size.y + 20.0));
    }
    painter.rect_filled(r, 7.0, SURFACE3);
    painter.rect_stroke(r, 7.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
    painter.galley(r.min + vec2(7.0, 4.0), galley, TEXT);
}
