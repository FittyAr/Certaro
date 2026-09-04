use printpdf::{Color, Mm, Point, Pt, Rgb as PdfRgb};
use super::super::theme::Rgb;

pub(super) const LINE_HEIGHT: f32 = 1.2;
pub(super) const AVG_CHAR_WIDTH: f32 = 0.5;
pub(super) const AVG_CHAR_WIDTH_BOLD: f32 = 0.54;

pub(super) fn point(x: f32, y: f32) -> Point {
    Point::new(Mm::from(Pt(x)), Mm::from(Pt(y)))
}

pub(super) fn color_of(rgb: Rgb) -> Color {
    Color::Rgb(PdfRgb::new(rgb.0, rgb.1, rgb.2, None))
}
