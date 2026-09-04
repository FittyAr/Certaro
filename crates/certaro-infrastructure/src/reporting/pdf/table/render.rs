use super::cell::{Row, Width};
use super::super::canvas::{Canvas, TextSpec};
use super::super::theme::Rgb;

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
