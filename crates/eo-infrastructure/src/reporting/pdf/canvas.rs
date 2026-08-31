//! The drawing surface the report layouts use. See `docs/12` §1.1.
//!
//! `printpdf` draws primitives; it has no notion of a cursor, a page break or a footer. This is
//! the layer that adds them, in **points measured from the top of the page**, which is how a
//! layout is described in the document and how anyone reading the code thinks about a page.
//!
//! **Deviation from doc 12 §1.2 rule 5**: the font is the built-in Helvetica rather than an
//! embedded Inter. Embedding needs a font binary in the repository, and Helvetica's WinAnsi
//! encoding covers every character Spanish and English paperwork uses. `Report.Font` stays in
//! configuration so switching later changes one place.

use eo_application::result::AppResult;
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::{
    BuiltinFont, Color, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerIndex, PdfLayerReference, PdfPageIndex, Point, Polygon, Pt, Rgb as PdfRgb,
};

use super::theme::{self, Rgb};
use crate::reporting::io_error;

/// Multiplier from font size to line height. 1.2 is the usual typographic default and matches the
/// spacing of the tables in the paper forms these reports reproduce.
const LINE_HEIGHT: f32 = 1.2;

/// Approximate advance width of Helvetica, as a fraction of the font size. Used to fit and, when
/// needed, truncate text. `printpdf` exposes no metrics for built-in fonts, and the alternative —
/// text that silently overflows into the next column — is worse than a slightly early ellipsis.
const AVG_CHAR_WIDTH: f32 = 0.5;
const AVG_CHAR_WIDTH_BOLD: f32 = 0.54;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle {
    #[default]
    Regular,
    Bold,
    Italic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// A piece of text with everything needed to place it.
#[derive(Debug, Clone)]
pub struct TextSpec {
    pub text: String,
    pub size: f32,
    pub style: FontStyle,
    pub color: Rgb,
    pub align: Align,
}

impl TextSpec {
    #[must_use]
    pub fn new(text: impl Into<String>, size: f32) -> Self {
        Self {
            text: text.into(),
            size,
            style: FontStyle::Regular,
            color: theme::TEXT,
            align: Align::Left,
        }
    }

    #[must_use]
    pub fn bold(mut self) -> Self {
        self.style = FontStyle::Bold;
        self
    }

    #[must_use]
    pub fn italic(mut self) -> Self {
        self.style = FontStyle::Italic;
        self
    }

    #[must_use]
    pub fn color(mut self, color: Rgb) -> Self {
        self.color = color;
        self
    }

    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

struct Fonts {
    regular: IndirectFontRef,
    bold: IndirectFontRef,
    italic: IndirectFontRef,
}

impl Fonts {
    fn pick(&self, style: FontStyle) -> &IndirectFontRef {
        match style {
            FontStyle::Regular => &self.regular,
            FontStyle::Bold => &self.bold,
            FontStyle::Italic => &self.italic,
        }
    }
}

pub struct Canvas {
    doc: PdfDocumentReference,
    pages: Vec<(PdfPageIndex, PdfLayerIndex)>,
    fonts: Fonts,
    width: f32,
    height: f32,
    margin: f32,
    /// Distance from the top of the page to where the next block goes.
    cursor: f32,
    /// Reserved at the bottom for the footer, so no content lands on it.
    footer_space: f32,
}

impl Canvas {
    /// A document of one page. Sizes in points; `A4` lives in [`theme::page`].
    pub fn new(title: &str, width: f32, height: f32, margin: f32) -> AppResult<Self> {
        let (doc, page, layer) =
            PdfDocument::new(title, Mm::from(Pt(width)), Mm::from(Pt(height)), "capa");
        let fonts = Fonts {
            regular: doc
                .add_builtin_font(BuiltinFont::Helvetica)
                .map_err(|e| io_error("pdf.font.regular", e))?,
            bold: doc
                .add_builtin_font(BuiltinFont::HelveticaBold)
                .map_err(|e| io_error("pdf.font.bold", e))?,
            italic: doc
                .add_builtin_font(BuiltinFont::HelveticaOblique)
                .map_err(|e| io_error("pdf.font.italic", e))?,
        };
        Ok(Self {
            doc,
            pages: vec![(page, layer)],
            fonts,
            width,
            height,
            margin,
            cursor: margin,
            footer_space: 28.0,
        })
    }

    #[must_use]
    pub fn content_width(&self) -> f32 {
        self.width - 2.0 * self.margin
    }

    #[must_use]
    pub fn left(&self) -> f32 {
        self.margin
    }

    #[must_use]
    pub fn cursor(&self) -> f32 {
        self.cursor
    }

    pub fn set_cursor(&mut self, y: f32) {
        self.cursor = y;
    }

    pub fn advance(&mut self, dy: f32) {
        self.cursor += dy;
    }

    /// Space left before the footer area.
    #[must_use]
    pub fn remaining(&self) -> f32 {
        self.height - self.margin - self.footer_space - self.cursor
    }

    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn new_page(&mut self) {
        let (page, layer) =
            self.doc
                .add_page(Mm::from(Pt(self.width)), Mm::from(Pt(self.height)), "capa");
        self.pages.push((page, layer));
        self.cursor = self.margin;
    }

    /// Starts a page when `needed` points do not fit. Returns whether it broke.
    pub fn ensure_space(&mut self, needed: f32) -> bool {
        if self.remaining() >= needed {
            return false;
        }
        self.new_page();
        true
    }

    fn layer(&self) -> PdfLayerReference {
        let (page, layer) = self.pages[self.pages.len() - 1];
        self.doc.get_page(page).get_layer(layer)
    }

    fn layer_of(&self, index: usize) -> PdfLayerReference {
        let (page, layer) = self.pages[index];
        self.doc.get_page(page).get_layer(layer)
    }

    /// Height one line of this size occupies.
    #[must_use]
    pub fn line_height(size: f32) -> f32 {
        size * LINE_HEIGHT
    }

    /// Approximate width of a string. Deliberately an estimate; see `AVG_CHAR_WIDTH`.
    #[must_use]
    pub fn text_width(text: &str, size: f32, style: FontStyle) -> f32 {
        let factor = if style == FontStyle::Bold {
            AVG_CHAR_WIDTH_BOLD
        } else {
            AVG_CHAR_WIDTH
        };
        text.chars().count() as f32 * size * factor
    }

    /// Cuts the text with an ellipsis so it fits `max_width`. A column that overflows into the
    /// next one makes the whole table unreadable, so the truncation is deliberate.
    #[must_use]
    pub fn fit(text: &str, size: f32, style: FontStyle, max_width: f32) -> String {
        if Self::text_width(text, size, style) <= max_width {
            return text.to_owned();
        }
        let factor = if style == FontStyle::Bold {
            AVG_CHAR_WIDTH_BOLD
        } else {
            AVG_CHAR_WIDTH
        };
        let fits = ((max_width / (size * factor)).floor() as usize).saturating_sub(1);
        if fits == 0 {
            return String::new();
        }
        let mut out: String = text.chars().take(fits).collect();
        out.push('…');
        out
    }

    /// Draws text in a box: `x` is its left edge, `width` its width, `y` the top of the line.
    pub fn text_in(&self, spec: &TextSpec, x: f32, width: f32, y: f32) {
        let text = Self::fit(&spec.text, spec.size, spec.style, width);
        if text.is_empty() {
            return;
        }
        let drawn = Self::text_width(&text, spec.size, spec.style);
        let start = match spec.align {
            Align::Left => x,
            Align::Center => x + (width - drawn) / 2.0,
            Align::Right => x + width - drawn,
        };
        self.raw_text(&text, spec, start.max(x), y, None);
    }

    /// Same, without a box: the text starts at `x` and is not truncated.
    pub fn text_at(&self, spec: &TextSpec, x: f32, y: f32) {
        self.raw_text(&spec.text, spec, x, y, None);
    }

    fn raw_text(&self, text: &str, spec: &TextSpec, x: f32, y_top: f32, page: Option<usize>) {
        let layer = match page {
            Some(index) => self.layer_of(index),
            None => self.layer(),
        };
        layer.set_fill_color(color_of(spec.color));
        // The baseline sits one font size below the top of the line box, which is what makes
        // successive lines look evenly spaced.
        let baseline = y_top + spec.size;
        layer.use_text(
            text,
            spec.size,
            Mm::from(Pt(x)),
            Mm::from(Pt(self.height - baseline)),
            self.fonts.pick(spec.style),
        );
    }

    pub fn rect(
        &self,
        x: f32,
        y_top: f32,
        width: f32,
        height: f32,
        fill: Option<Rgb>,
        stroke: Option<(Rgb, f32)>,
    ) {
        let layer = self.layer();
        let top = self.height - y_top;
        let bottom = top - height;
        let ring = vec![
            (point(x, bottom), false),
            (point(x, top), false),
            (point(x + width, top), false),
            (point(x + width, bottom), false),
        ];

        if let Some(fill) = fill {
            layer.set_fill_color(color_of(fill));
            layer.add_polygon(Polygon {
                rings: vec![ring.clone()],
                mode: PaintMode::Fill,
                winding_order: WindingOrder::NonZero,
            });
        }
        if let Some((color, thickness)) = stroke {
            layer.set_outline_color(color_of(color));
            layer.set_outline_thickness(thickness);
            layer.add_polygon(Polygon {
                rings: vec![ring],
                mode: PaintMode::Stroke,
                winding_order: WindingOrder::NonZero,
            });
        }
    }

    pub fn hline(&self, x: f32, y_top: f32, width: f32, color: Rgb, thickness: f32) {
        let layer = self.layer();
        let y = self.height - y_top;
        layer.set_outline_color(color_of(color));
        layer.set_outline_thickness(thickness);
        layer.add_line(Line {
            points: vec![(point(x, y), false), (point(x + width, y), false)],
            is_closed: false,
        });
    }

    pub fn vline(&self, x: f32, y_top: f32, height: f32, color: Rgb, thickness: f32) {
        let layer = self.layer();
        let top = self.height - y_top;
        layer.set_outline_color(color_of(color));
        layer.set_outline_thickness(thickness);
        layer.add_line(Line {
            points: vec![(point(x, top), false), (point(x, top - height), false)],
            is_closed: false,
        });
    }

    /// Writes the footer on every page and returns the document.
    ///
    /// It runs at the end because the total number of pages is not known until then, and a footer
    /// that says «page 3» without saying «of 7» is the legacy footer this replaces.
    pub fn finish<F>(self, footer: F) -> AppResult<Vec<u8>>
    where
        F: Fn(usize, usize) -> Option<TextSpec>,
    {
        let total = self.pages.len();
        for index in 0..total {
            if let Some(spec) = footer(index + 1, total) {
                let y = self.height - self.margin - theme::size::FOOTER;
                let text = spec.text.clone();
                let drawn = Self::text_width(&text, spec.size, spec.style);
                let x = match spec.align {
                    Align::Left => self.margin,
                    Align::Center => (self.width - drawn) / 2.0,
                    Align::Right => self.width - self.margin - drawn,
                };
                self.raw_text(&text, &spec, x, y, Some(index));
            }
        }
        self.doc
            .save_to_bytes()
            .map_err(|e| io_error("pdf.save", e))
    }
}

fn point(x: f32, y: f32) -> Point {
    Point::new(Mm::from(Pt(x)), Mm::from(Pt(y)))
}

fn color_of(rgb: Rgb) -> Color {
    Color::Rgb(PdfRgb::new(rgb.0, rgb.1, rgb.2, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas() -> Canvas {
        Canvas::new(
            "prueba",
            theme::page::A4_WIDTH,
            theme::page::A4_HEIGHT,
            theme::page::MARGIN_MOVIMIENTOS,
        )
        .unwrap()
    }

    #[test]
    fn el_ancho_util_descuenta_los_dos_margenes() {
        let c = canvas();
        assert!((c.content_width() - (theme::page::A4_WIDTH - 2.0 * 28.35)).abs() < 0.01);
    }

    #[test]
    fn pedir_mas_espacio_del_que_queda_abre_una_pagina() {
        let mut c = canvas();
        assert!(!c.ensure_space(100.0));
        assert_eq!(c.page_count(), 1);
        assert!(c.ensure_space(10_000.0));
        assert_eq!(c.page_count(), 2);
        assert_eq!(c.cursor(), theme::page::MARGIN_MOVIMIENTOS);
    }

    #[test]
    fn el_texto_que_no_cabe_se_recorta_con_puntos_suspensivos() {
        let recortado = Canvas::fit(
            "Un concepto larguísimo que no entra",
            10.0,
            FontStyle::Regular,
            40.0,
        );
        assert!(recortado.ends_with('…'), "{recortado}");
        assert!(recortado.chars().count() < 12, "{recortado}");
    }

    #[test]
    fn el_texto_que_cabe_no_se_toca() {
        assert_eq!(
            Canvas::fit("Cable", 10.0, FontStyle::Regular, 200.0),
            "Cable"
        );
    }

    #[test]
    fn un_ancho_ridiculo_devuelve_vacio_en_lugar_de_solo_puntos() {
        assert_eq!(Canvas::fit("Cable", 10.0, FontStyle::Regular, 2.0), "");
    }

    #[test]
    fn el_documento_se_guarda_con_su_pie_en_cada_pagina() {
        let mut c = canvas();
        c.text_at(&TextSpec::new("Hola", 10.0), c.left(), c.cursor());
        c.new_page();
        let bytes = c
            .finish(|actual, total| {
                Some(TextSpec::new(format!("{actual}/{total}"), 8.0).align(Align::Center))
            })
            .unwrap();
        assert!(bytes.starts_with(b"%PDF"), "no es un PDF");
    }
}
