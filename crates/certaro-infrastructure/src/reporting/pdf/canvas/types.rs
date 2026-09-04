use printpdf::PdfFontHandle;
use super::super::theme::{self, Rgb};

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

pub(super) struct Fonts {
    pub(super) regular: PdfFontHandle,
    pub(super) bold: PdfFontHandle,
    pub(super) italic: PdfFontHandle,
}

impl Fonts {
    pub(super) fn pick(&self, style: FontStyle) -> PdfFontHandle {
        match style {
            FontStyle::Regular => self.regular.clone(),
            FontStyle::Bold => self.bold.clone(),
            FontStyle::Italic => self.italic.clone(),
        }
    }
}
