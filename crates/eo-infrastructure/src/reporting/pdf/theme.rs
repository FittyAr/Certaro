//! Sizes and colours of the reports, in one place. See `docs/12` §1.2 rule 5.
//!
//! The legacy generator declared no font at all, so the PDF came out in the library's default,
//! the Word document in Calibri and the spreadsheet in Calibri 11: three typefaces for the same
//! data. Here every size and colour of every report is named here and nowhere else.

/// A colour as its three components, 0..=1, which is what the PDF library takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb(pub f32, pub f32, pub f32);

impl Rgb {
    #[must_use]
    pub const fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}

pub const BLACK: Rgb = Rgb(0.0, 0.0, 0.0);
pub const WHITE: Rgb = Rgb(1.0, 1.0, 1.0);
/// Body text: near-black, because pure black on white prints harsher than it reads.
pub const TEXT: Rgb = Rgb::from_u8(38, 38, 38);
pub const MUTED: Rgb = Rgb::from_u8(115, 115, 115);
pub const PRIMARY: Rgb = Rgb::from_u8(21, 94, 117);
pub const POSITIVE: Rgb = Rgb::from_u8(21, 128, 61);
pub const NEGATIVE: Rgb = Rgb::from_u8(185, 28, 28);
pub const LINE: Rgb = Rgb::from_u8(212, 212, 212);
pub const ZEBRA: Rgb = Rgb::from_u8(247, 247, 247);
pub const TOTAL_FILL: Rgb = Rgb::from_u8(242, 242, 242);
/// The two greens of the certificate's header, which reproduce the paper form it replaces.
pub const REPORT_HEADER: Rgb = Rgb::from_u8(21, 84, 56);
pub const REPORT_SUBHEADER: Rgb = Rgb::from_u8(214, 232, 220);
pub const TOTAL_A_FACTURAR_FILL: Rgb = Rgb::from_u8(232, 245, 237);

/// Font sizes, in points.
pub mod size {
    pub const TITLE: f32 = 20.0;
    pub const TITLE_LIQUIDACION: f32 = 24.0;
    pub const SUBTITLE: f32 = 8.0;
    pub const BODY: f32 = 10.0;
    pub const BODY_LIQUIDACION: f32 = 11.0;
    pub const BODY_CERTIFICADO: f32 = 9.0;
    pub const SECTION: f32 = 10.0;
    pub const EMPLEADO: f32 = 14.0;
    pub const TOTAL: f32 = 14.0;
    pub const FOOTER: f32 = 8.0;
    pub const CERT_HEADER: f32 = 8.0;
    pub const CERT_SUBHEADER: f32 = 7.0;
    pub const CERT_TOTAL: f32 = 11.0;
}

/// Page geometry, in points. A4 is 595.28 × 841.89 pt.
pub mod page {
    pub const A4_WIDTH: f32 = 595.28;
    pub const A4_HEIGHT: f32 = 841.89;
    /// 1 cm.
    pub const MARGIN_MOVIMIENTOS: f32 = 28.35;
    /// 1.5 cm.
    pub const MARGIN_LIQUIDACION: f32 = 42.52;
    /// 1 cm.
    pub const MARGIN_CERTIFICADO: f32 = 28.35;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_componentes_quedan_normalizados() {
        let Rgb(r, g, b) = Rgb::from_u8(255, 128, 0);
        assert!((r - 1.0).abs() < f32::EPSILON);
        assert!((g - 0.502).abs() < 0.001);
        assert!(b.abs() < f32::EPSILON);
    }
}
