//! The table engine of the reports. See `docs/12-reportes-y-exportaciones.md` §1.1.
//!
//! What the layouts need and no PDF crate provides: relative and fixed column widths, cells merged
//! across columns and across rows, per-cell style, a header repeated on every page, and totals
//! pinned to the end.
//!
//! Everything is measured in points from the top of the page, like [`super::canvas`].

use super::canvas::{Align, Canvas, FontStyle, TextSpec};
use super::theme::{self, Rgb};

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

pub struct Table {
    pub widths: Vec<Width>,
    /// Repeated at the top of every page.
    pub header: Vec<Row>,
    pub rows: Vec<Row>,
    /// Totals, kept together with the last data row.
    pub footer: Vec<Row>,
    pub font_size: f32,
    pub padding_v: f32,
    pub padding_h: f32,
    /// Alternating background for the data rows.
    pub zebra: Option<Rgb>,
}

impl Table {
    #[must_use]
    pub fn new(widths: Vec<Width>, font_size: f32) -> Self {
        Self {
            widths,
            header: Vec::new(),
            rows: Vec::new(),
            footer: Vec::new(),
            font_size,
            padding_v: 5.0,
            padding_h: 3.0,
            zebra: None,
        }
    }

    /// The x offset and the width of each column, for the available width.
    #[must_use]
    pub fn geometry(&self, available: f32) -> Vec<(f32, f32)> {
        let fixed: f32 = self
            .widths
            .iter()
            .map(|w| match w {
                Width::Fixed(pt) => *pt,
                Width::Relative(_) => 0.0,
            })
            .sum();
        let shares: f32 = self
            .widths
            .iter()
            .map(|w| match w {
                Width::Relative(share) => *share,
                Width::Fixed(_) => 0.0,
            })
            .sum();
        // Fixed columns win when the page is too narrow for both: they reproduce a printed form,
        // while the relative ones only hold text that can be truncated.
        let flexible = (available - fixed).max(0.0);

        let mut out = Vec::with_capacity(self.widths.len());
        let mut x = 0.0;
        for width in &self.widths {
            let w = match width {
                Width::Fixed(pt) => *pt,
                Width::Relative(share) if shares > 0.0 => flexible * share / shares,
                Width::Relative(_) => 0.0,
            };
            out.push((x, w));
            x += w;
        }
        out
    }

    fn row_height(&self, row: &Row) -> f32 {
        let size = row
            .cells
            .iter()
            .filter_map(|c| c.size)
            .fold(self.font_size, f32::max);
        Canvas::line_height(size) + 2.0 * row.padding_v.unwrap_or(self.padding_v)
    }

    /// Draws the table, breaking pages and repeating the header. Returns the y the cursor ends at.
    pub fn render(&self, canvas: &mut Canvas) {
        let left = canvas.left();
        let geometry = self.geometry(canvas.content_width());
        self.render_block(canvas, left, &geometry, &self.header);

        for (index, row) in self.rows.iter().enumerate() {
            let height = self.row_height(row);
            if canvas.ensure_space(height) {
                // A page whose table has no column headings is unreadable, so they come along.
                self.render_block(canvas, left, &geometry, &self.header);
            }
            let zebra = self.zebra.filter(|_| index % 2 == 1 && row.fill.is_none());
            self.render_row(canvas, left, &geometry, row, zebra);
        }

        let footer_height: f32 = self.footer.iter().map(|r| self.row_height(r)).sum();
        if !self.footer.is_empty() && canvas.ensure_space(footer_height) {
            self.render_block(canvas, left, &geometry, &self.header);
        }
        self.render_block(canvas, left, &geometry, &self.footer);
    }

    /// A group of rows that must stay together: the header and the totals.
    fn render_block(&self, canvas: &mut Canvas, left: f32, geometry: &[(f32, f32)], rows: &[Row]) {
        // Columns still covered by a `rowspan` from an earlier row, and for how many more rows.
        let mut occupied: Vec<usize> = vec![0; geometry.len()];
        for row in rows {
            self.render_row_with_occupancy(canvas, left, geometry, row, None, &mut occupied);
        }
    }

    fn render_row(
        &self,
        canvas: &mut Canvas,
        left: f32,
        geometry: &[(f32, f32)],
        row: &Row,
        zebra: Option<Rgb>,
    ) {
        let mut occupied: Vec<usize> = vec![0; geometry.len()];
        self.render_row_with_occupancy(canvas, left, geometry, row, zebra, &mut occupied);
    }

    fn render_row_with_occupancy(
        &self,
        canvas: &mut Canvas,
        left: f32,
        geometry: &[(f32, f32)],
        row: &Row,
        zebra: Option<Rgb>,
        occupied: &mut [usize],
    ) {
        let height = self.row_height(row);
        let top = canvas.cursor();
        let padding_v = row.padding_v.unwrap_or(self.padding_v);

        if let Some(color) = row.fill.or(zebra) {
            canvas.rect(left, top, canvas.content_width(), height, Some(color), None);
        }

        let mut column = 0;
        for cell in &row.cells {
            // Skip whatever a `rowspan` above is still covering.
            while column < occupied.len() && occupied[column] > 0 {
                column += 1;
            }
            if column >= geometry.len() {
                break;
            }

            let span = cell.colspan.min(geometry.len() - column);
            let (x_offset, _) = geometry[column];
            let width: f32 = geometry[column..column + span].iter().map(|(_, w)| w).sum();
            let x = left + x_offset;
            let cell_height = if cell.rowspan > 1 {
                height * cell.rowspan as f32
            } else {
                height
            };

            if let Some(fill) = cell.fill {
                canvas.rect(x, top, width, cell_height, Some(fill), None);
            }
            if let Some(border) = row.grid {
                canvas.rect(
                    x,
                    top,
                    width,
                    cell_height,
                    None,
                    Some((border.color, border.thickness)),
                );
            }

            let size = cell.size.unwrap_or(self.font_size);
            // A cell spanning rows is centred over the whole span, which is what makes the
            // certificate's two-row header read as one column heading.
            let text_top = top + (cell_height - Canvas::line_height(size)) / 2.0;
            let spec = TextSpec {
                text: cell.text.clone(),
                size,
                style: cell.style,
                color: cell.color,
                align: cell.align,
            };
            canvas.text_in(
                &spec,
                x + self.padding_h,
                (width - 2.0 * self.padding_h).max(0.0),
                if cell.rowspan > 1 {
                    text_top
                } else {
                    top + padding_v
                },
            );

            if cell.rowspan > 1 {
                for slot in occupied.iter_mut().skip(column).take(span) {
                    *slot = cell.rowspan - 1;
                }
            }
            column += span;
        }

        if let Some(border) = row.border_bottom {
            canvas.hline(
                left,
                top + height,
                canvas.content_width(),
                border.color,
                border.thickness,
            );
        }

        for slot in occupied.iter_mut() {
            *slot = slot.saturating_sub(1);
        }
        canvas.advance(height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabla(widths: Vec<Width>) -> Table {
        Table::new(widths, 10.0)
    }

    #[test]
    fn las_columnas_relativas_reparten_el_ancho_en_proporcion() {
        let t = tabla(vec![Width::Relative(1.0), Width::Relative(3.0)]);
        let g = t.geometry(400.0);
        assert!((g[0].1 - 100.0).abs() < 0.01);
        assert!((g[1].1 - 300.0).abs() < 0.01);
        assert!((g[1].0 - 100.0).abs() < 0.01);
    }

    #[test]
    fn las_columnas_constantes_conservan_sus_puntos() {
        let t = tabla(vec![
            Width::Relative(1.0),
            Width::Fixed(80.0),
            Width::Fixed(80.0),
        ]);
        let g = t.geometry(400.0);
        assert!((g[0].1 - 240.0).abs() < 0.01);
        assert!((g[1].1 - 80.0).abs() < 0.01);
        assert!((g[2].0 - 320.0).abs() < 0.01);
    }

    #[test]
    fn si_las_constantes_no_caben_las_relativas_quedan_en_cero_y_no_en_negativo() {
        let t = tabla(vec![Width::Relative(1.0), Width::Fixed(500.0)]);
        let g = t.geometry(400.0);
        assert_eq!(g[0].1, 0.0);
        assert!(g[1].1 > 0.0);
    }

    #[test]
    fn la_geometria_devuelve_una_entrada_por_columna() {
        let t = tabla(vec![Width::Fixed(30.0); 9]);
        assert_eq!(t.geometry(600.0).len(), 9);
    }

    #[test]
    fn una_tabla_larga_ocupa_mas_de_una_pagina_y_repite_el_encabezado() {
        let mut canvas = Canvas::new(
            "t",
            theme::page::A4_WIDTH,
            theme::page::A4_HEIGHT,
            theme::page::MARGIN_MOVIMIENTOS,
        )
        .unwrap();
        let mut t = tabla(vec![Width::Relative(1.0), Width::Relative(1.0)]);
        t.header = vec![
            Row::new(vec![Cell::new("Fecha").bold(), Cell::new("Total").bold()])
                .border_bottom(Border::hairline()),
        ];
        t.rows = (0..120)
            .map(|i| Row::new(vec![Cell::new(format!("fila {i}")), Cell::new("100,00")]))
            .collect();
        t.render(&mut canvas);
        assert!(canvas.page_count() > 1, "no paginó");
    }

    #[test]
    fn una_celda_combinada_no_desborda_la_cantidad_de_columnas() {
        let mut canvas = Canvas::new("t", 400.0, 400.0, 20.0).unwrap();
        let mut t = tabla(vec![Width::Relative(1.0); 3]);
        t.rows = vec![Row::new(vec![Cell::new("todo").colspan(9)])];
        t.render(&mut canvas);
        assert_eq!(canvas.page_count(), 1);
    }
}
