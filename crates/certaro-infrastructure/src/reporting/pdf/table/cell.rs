use super::super::canvas::{Align, FontStyle};
use super::super::theme::{self, Rgb};

/// How wide a column is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Width {
    /// A share of whatever is left after the fixed columns.
    Relative(f32),
    /// Points, regardless of the page. The certificate needs these so its numeric columns keep the
    /// width of the paper form.
    Fixed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    pub color: Rgb,
    pub thickness: f32,
}

impl Border {
    #[must_use]
    pub const fn new(color: Rgb, thickness: f32) -> Self {
        Self { color, thickness }
    }

    #[must_use]
    pub const fn thin() -> Self {
        Self::new(theme::LINE, 1.0)
    }

    #[must_use]
    pub const fn hairline() -> Self {
        Self::new(theme::BLACK, 0.5)
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub text: String,
    pub align: Align,
    pub style: FontStyle,
    pub color: Rgb,
    pub size: Option<f32>,
    /// Columns this cell covers.
    pub colspan: usize,
    /// Rows it covers. Only the first row declares it; the rows below skip those columns.
    pub rowspan: usize,
    pub fill: Option<Rgb>,
}

impl Cell {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            align: Align::Left,
            style: FontStyle::Regular,
            color: theme::TEXT,
            size: None,
            colspan: 1,
            rowspan: 1,
            fill: None,
        }
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::new("")
    }

    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
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
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    #[must_use]
    pub fn colspan(mut self, span: usize) -> Self {
        self.colspan = span.max(1);
        self
    }

    #[must_use]
    pub fn rowspan(mut self, span: usize) -> Self {
        self.rowspan = span.max(1);
        self
    }

    #[must_use]
    pub fn fill(mut self, color: Rgb) -> Self {
        self.fill = Some(color);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct Row {
    pub cells: Vec<Cell>,
    pub fill: Option<Rgb>,
    pub border_bottom: Option<Border>,
    /// Border around every cell of the row, which is what the certificate's grid is made of.
    pub grid: Option<Border>,
    /// Vertical padding, overriding the table's.
    pub padding_v: Option<f32>,
}

impl Row {
    #[must_use]
    pub fn new(cells: Vec<Cell>) -> Self {
        Self {
            cells,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn fill(mut self, color: Rgb) -> Self {
        self.fill = Some(color);
        self
    }

    #[must_use]
    pub fn border_bottom(mut self, border: Border) -> Self {
        self.border_bottom = Some(border);
        self
    }

    #[must_use]
    pub fn grid(mut self, border: Border) -> Self {
        self.grid = Some(border);
        self
    }

    #[must_use]
    pub fn padding_v(mut self, padding: f32) -> Self {
        self.padding_v = Some(padding);
        self
    }
}
