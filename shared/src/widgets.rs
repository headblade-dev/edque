use eframe::egui::{
    Ui, Color32, Label, 
    RichText, FontFamily, Vec2, 
    Align2, FontId, Response, 
    Stroke, StrokeKind, Sense,
    Frame, Margin, TextEdit,
};

///
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
    dims: (f32, f32),
    radius: u8,
    bg: Color32,
    fg: Color32,
    stroke_color: Option<Color32>,
    text: &str,
    font: &str,
    size: i32,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(dims.0, dims.1), Sense::click());

    ui.painter().rect_filled(rect, radius, bg);

    if let Some(stroke_color) = stroke_color {
        ui.painter().rect_stroke(
            rect, radius,
            Stroke::new(1.0, stroke_color),
            StrokeKind::Outside,
        );
    }

    #[rustfmt::skip]
    ui.painter().text(
        rect.center(), Align2::CENTER_CENTER,
        text,
        FontId::new(size as f32, FontFamily::Name(font.into())),
        fg,
    );
    resp
}

///
/// Draws an input area. Not for updates on loosing focus
///
pub fn add_input(
    ui: &mut Ui,
    dims: (f32, f32),
    radius: u8,
    margin: Margin,
    stroke: Stroke,
    buf: &mut String,
    hint: &str,
    hint_color: Color32,
    hint_size: i32,
) -> Response {
    let resp = Frame::new()
        .corner_radius(radius)
        .inner_margin(margin)
        .stroke(stroke)
        .show(ui, |ui| {
            ui.set_width(dims.0);
            ui.add_sized(
                Vec2::new(dims.0, dims.1),
                TextEdit::singleline(buf)
                    .font(
                        FontId::new(
                            hint_size as f32, 
                            FontFamily::Name("Inter".into())
                        )
                    )
                    .hint_text(
                        RichText::new(hint)
                            .family(FontFamily::Name("Inter".into()))
                            .size(hint_size as f32)
                            .color(hint_color),
                    )
                    .frame(Frame::NONE)
            )
        });

    resp.inner
}
