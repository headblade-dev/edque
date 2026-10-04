use eframe::egui::{
    Align2, Color32, FontFamily, FontId, Frame, Label, Margin, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
};

use crate::theme;

pub struct Input {
    pub dims: (f32, f32),
    pub radius: u8,
    pub margin: Margin,
    pub stroke: Stroke,
    pub buf: String,
    pub hint: String,
    pub hint_color: Color32,
    pub hint_size: i32,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            dims: (100.0, 100.0), 
            radius: 8, 
            margin: Margin::same(0), 
            stroke: Stroke { width: 1.0, color: theme::TEXT_DIM }, 
            buf: String::new(), 
            hint: "Enter your text...".to_string(), 
            hint_color: theme::TEXT_DIM, 
            hint_size: 16 
        }
    }
}

pub struct Button {
    pub dims: (f32, f32),
    pub radius: u8,
    pub bg: Color32,
    pub fg: Color32,
    pub stroke_color: Option<Color32>,
    pub text: String,
    pub font: String,
    pub size: i32,
}
impl Default for Button {
    fn default() -> Self {
        Self {
            dims: (300.0, 64.0),
            radius: 8,
            bg: theme::ACCENT,
            fg: theme::TEXT_ON_ACCENT,
            stroke_color: None,
            text: "Кнопка".to_string(),
            font: "Inter-SemiBold".to_string(),
            size: 16,
        }
    }
}

/// Draws a `Label` with `RichText` inside of it
/// 
pub fn rich_label(ui: &mut Ui, text: &str, size: u8, color: Color32, weight: u16) {
    ui.add(
        Label::new(
            RichText::new(text)
                .family(FontFamily::Name("Inter".into()))
                .size(size as f32)
                .color(color)
                .variation("wght", weight as f32)
        ).selectable(false)
    );
}

///
/// Draws a `Label` using given size with `RichText` inside of it
/// 
pub fn rich_label_sized(ui: &mut Ui, dims: (f32, f32), text: &str, size: u8, color: Color32, wght: u16) {  
    ui.add_sized(
        Vec2::new(dims.0, dims.1),
        Label::new(
            RichText::new(text)
                .family(FontFamily::Name("Inter".into()))
                .size(size as f32)
                .color(color)
                .variation("wght", wght as f32)
        ).selectable(false)
    );
}

///
/// Draws a button with given parameters
/// 
pub fn add_button(
    ui: &mut Ui,
    button: Button,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(button.dims.0, button.dims.1), Sense::click());

    ui.painter().rect_filled(rect, button.radius, button.bg);

    if let Some(stroke_color) = button.stroke_color {
        ui.painter().rect_stroke(
            rect, button.radius,
            Stroke::new(1.0, stroke_color),
            StrokeKind::Outside,
        );
    }

    #[rustfmt::skip]
    ui.painter().text(
        rect.center(), Align2::CENTER_CENTER,
        button.text,
        FontId::new(button.size as f32, FontFamily::Name(button.font.into())),
        button.fg,
    );
    resp
}

///
/// Draws an input area. Not for updates on loosing focus
///
pub fn add_input(
    ui: &mut Ui,
    input: &mut Input,
) -> Response {
    let resp = Frame::new()
        .corner_radius(input.radius)
        .inner_margin(input.margin)
        .stroke(input.stroke)
        .show(ui, |ui| {
            ui.set_width(input.dims.0);
            ui.add_sized(
                Vec2::new(input.dims.0, input.dims.1),
                TextEdit::singleline(&mut input.buf)
                    .font(
                        FontId::new(
                            input.hint_size as f32, 
                            FontFamily::Name("Inter".into())
                        )
                    )
                    .hint_text(
                        RichText::new(input.hint.clone())
                            .family(FontFamily::Name("Inter".into()))
                            .size(input.hint_size as f32)
                            .color(input.hint_color),
                    )
                    .frame(Frame::NONE)
            )
        });

    resp.inner
}
