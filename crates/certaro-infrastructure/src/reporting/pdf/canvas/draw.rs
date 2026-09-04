use printpdf::{Line, LinePoint, Op, Polygon, PolygonRing, Pt, TextItem, WindingOrder};
use super::theme::Rgb;
use super::types::TextSpec;
use super::helpers::{color_of, point};
use super::Canvas;

impl Canvas {
    pub(crate) fn current_ops(ops_ref: &mut [Vec<Op>]) -> &mut Vec<Op> {
        let len = ops_ref.len();
        if len == 0 {
            unreachable!("canvas always has at least one page")
        }
        &mut ops_ref[len - 1]
    }

    pub(crate) fn raw_text(&self, text: &str, spec: &TextSpec, x: f32, y_top: f32, page: Option<usize>) {
        let height = self.height;
        let font = self.fonts.pick(spec.style);
        let col = color_of(spec.color);
        let size = spec.size;
        let baseline = y_top + size;
        let y = height - baseline;
        let pos = point(x, y);
        let items = vec![TextItem::Text(text.to_owned())];
        let mut ops_ref = self.ops_per_page.borrow_mut();
        let ops: &mut Vec<Op> = if let Some(index) = page {
            &mut ops_ref[index]
        } else {
            Self::current_ops(&mut ops_ref)
        };
        ops.extend([
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

    pub fn rect(
        &self,
        x: f32,
        y_top: f32,
        width: f32,
        height: f32,
        fill: Option<Rgb>,
        stroke: Option<(Rgb, f32)>,
    ) {
        let h = self.height;
        let top = h - y_top;
        let bottom = top - height;
        let ring = vec![
            (point(x, bottom), false),
            (point(x, top), false),
            (point(x + width, top), false),
            (point(x + width, bottom), false),
        ];
        let mut ops_ref = self.ops_per_page.borrow_mut();
        let ops = Self::current_ops(&mut ops_ref);
        if let Some(fill) = fill {
            ops.push(Op::SetFillColor {
                col: color_of(fill),
            });
            ops.push(Op::DrawPolygon {
                polygon: Polygon {
                    rings: vec![PolygonRing {
                        points: ring
                            .clone()
                            .into_iter()
                            .map(|(p, bezier)| LinePoint { p, bezier })
                            .collect(),
                    }],
                    mode: printpdf::PaintMode::Fill,
                    winding_order: WindingOrder::NonZero,
                },
            });
        }
        if let Some((color, thickness)) = stroke {
            ops.push(Op::SetOutlineColor {
                col: color_of(color),
            });
            ops.push(Op::SetOutlineThickness { pt: Pt(thickness) });
            ops.push(Op::DrawPolygon {
                polygon: Polygon {
                    rings: vec![PolygonRing {
                        points: ring
                            .into_iter()
                            .map(|(p, bezier)| LinePoint { p, bezier })
                            .collect(),
                    }],
                    mode: printpdf::PaintMode::Stroke,
                    winding_order: WindingOrder::NonZero,
                },
            });
        }
    }

    pub fn hline(&self, x: f32, y_top: f32, width: f32, color: Rgb, thickness: f32) {
        let y = self.height - y_top;
        let mut ops_ref = self.ops_per_page.borrow_mut();
        let ops = Self::current_ops(&mut ops_ref);
        ops.push(Op::SetOutlineColor {
            col: color_of(color),
        });
        ops.push(Op::SetOutlineThickness { pt: Pt(thickness) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: point(x, y),
                        bezier: false,
                    },
                    LinePoint {
                        p: point(x + width, y),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }

    pub fn vline(&self, x: f32, y_top: f32, height: f32, color: Rgb, thickness: f32) {
        let top = self.height - y_top;
        let mut ops_ref = self.ops_per_page.borrow_mut();
        let ops = Self::current_ops(&mut ops_ref);
        ops.push(Op::SetOutlineColor {
            col: color_of(color),
        });
        ops.push(Op::SetOutlineThickness { pt: Pt(thickness) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: point(x, top),
                        bezier: false,
                    },
                    LinePoint {
                        p: point(x, top - height),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }
}
