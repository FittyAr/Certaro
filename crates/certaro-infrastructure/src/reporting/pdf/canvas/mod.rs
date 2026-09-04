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

use std::cell::{Cell, RefCell};

use certaro_application::result::AppResult;
use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Pt, TextItem,
};

use super::theme;

mod draw;
mod helpers;
mod types;

#[cfg(test)]
mod tests;

use helpers::*;
pub use types::*;

pub struct Canvas {
    pub(crate) doc: PdfDocument,
    pub(crate) ops_per_page: RefCell<Vec<Vec<Op>>>,
    pub(crate) fonts: Fonts,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) margin: f32,
    pub(crate) cursor: Cell<f32>,
    pub(crate) footer_space: f32,
}

impl Canvas {
    /// A document of one page. Sizes in points; `A4` lives in [`theme::page`].
    pub fn new(title: &str, width: f32, height: f32, margin: f32) -> AppResult<Self> {
        let doc = PdfDocument::new(title);
        let fonts = Fonts {
            regular: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
            bold: PdfFontHandle::Builtin(BuiltinFont::HelveticaBold),
            italic: PdfFontHandle::Builtin(BuiltinFont::HelveticaOblique),
        };
        Ok(Self {
            doc,
            ops_per_page: RefCell::new(vec![Vec::new()]),
            fonts,
            width,
            height,
            margin,
            cursor: Cell::new(margin),
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
        self.cursor.get()
    }

    pub fn set_cursor(&self, y: f32) {
        self.cursor.set(y);
    }

    pub fn advance(&self, dy: f32) {
        self.cursor.set(self.cursor.get() + dy);
    }

    /// Space left before the footer area.
    #[must_use]
    pub fn remaining(&self) -> f32 {
        self.height - self.margin - self.footer_space - self.cursor.get()
    }

    #[must_use]
    pub fn page_count(&self) -> usize {
        self.ops_per_page.borrow().len()
    }

    pub fn new_page(&self) {
        self.ops_per_page.borrow_mut().push(Vec::new());
        self.cursor.set(self.margin);
    }

    /// Starts a page when `needed` points do not fit. Returns whether it broke.
    pub fn ensure_space(&self, needed: f32) -> bool {
        if self.remaining() >= needed {
            return false;
        }
        self.new_page();
        true
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

    /// Writes the footer on every page and returns the document.
    ///
    /// It runs at the end because the total number of pages is not known until then, and a footer
    /// that says «page 3» without saying «of 7» is the legacy footer this replaces.
    pub fn finish<F>(mut self, footer: F) -> AppResult<Vec<u8>>
    where
        F: Fn(usize, usize) -> Option<TextSpec>,
    {
        let total = self.ops_per_page.borrow().len();
        // Collect footer ops to avoid borrow conflicts: we need to push to each page
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
                let spec_clone = TextSpec {
                    text,
                    size: spec.size,
                    style: spec.style,
                    color: spec.color,
                    align: spec.align,
                };
                // raw_text needs &self, but we are in finish taking self; we can call helper directly
                // Duplicate logic to avoid borrowing self.ops_per_page while iterating
                let font = self.fonts.pick(spec_clone.style);
                let col = color_of(spec_clone.color);
                let size = spec_clone.size;
                let baseline = y + size;
                let yy = self.height - baseline;
                let pos = point(x, yy);
                let items = vec![TextItem::Text(spec_clone.text)];
                self.ops_per_page.borrow_mut()[index].extend([
                    Op::SetFillColor { col },
                    Op::StartTextSection,
                    Op::SetTextCursor { pos },
                    Op::SetLineHeight { lh: Pt(size) },
                    Op::SetFont {
                        font,
                        size: Pt(size),
                    },
                    Op::ShowText { items },
                    Op::EndTextSection,
                ]);
            }
        }
        let pages: Vec<PdfPage> = self
            .ops_per_page
            .into_inner()
            .into_iter()
            .map(|ops| PdfPage::new(Mm::from(Pt(self.width)), Mm::from(Pt(self.height)), ops))
            .collect();
        let mut warnings = Vec::new();
        let bytes = self
            .doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut warnings);
        Ok(bytes)
    }
}
