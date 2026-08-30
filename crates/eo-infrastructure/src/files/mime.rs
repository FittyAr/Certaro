//! The allowed extensions and their MIME types. See `docs/13-servicios-externos-y-archivos.md` §1.4.
//!
//! One table, here, rather than the same list repeated at each call site. An extension outside it is
//! **refused**: the legacy code labelled anything unknown `application/octet-stream` and stored it,
//! which is how a renamed executable got into the user's data directory.

/// Extension without the dot, lowercase, and the MIME type it maps to.
pub const TIPOS: [(&str, &str); 16] = [
    ("pdf", "application/pdf"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png", "image/png"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("heic", "image/heic"),
    ("txt", "text/plain"),
    ("csv", "text/csv"),
    ("doc", "application/msword"),
    (
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ),
    ("xls", "application/vnd.ms-excel"),
    (
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ),
    ("odt", "application/vnd.oasis.opendocument.text"),
    ("ods", "application/vnd.oasis.opendocument.spreadsheet"),
    ("zip", "application/zip"),
];

/// The MIME type of an extension, or `None` when it is not on the list.
#[must_use]
pub fn de_extension(extension: &str) -> Option<&'static str> {
    let lower = extension.to_lowercase();
    TIPOS
        .iter()
        .find(|(ext, _)| *ext == lower)
        .map(|(_, mime)| *mime)
}

/// Whether the leading bytes are consistent with the extension.
///
/// Only the formats with a stable signature are checked; for the rest — plain text, CSV — there is
/// nothing to check and the answer is yes. This is not a real defence, but it catches the obvious
/// case of an executable renamed to `.pdf`, which is what §1.3 asks for.
#[must_use]
pub fn contenido_coincide(extension: &str, bytes: &[u8]) -> bool {
    let firma: &[&[u8]] = match extension.to_lowercase().as_str() {
        "pdf" => &[b"%PDF-"],
        "png" => &[&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]],
        "jpg" | "jpeg" => &[&[0xFF, 0xD8, 0xFF]],
        "gif" => &[b"GIF87a", b"GIF89a"],
        "webp" => &[b"RIFF"],
        // OOXML and ODF are zip containers, so all four share the signature.
        "zip" | "docx" | "xlsx" | "odt" | "ods" => &[b"PK\x03\x04", b"PK\x05\x06", b"PK\x07\x08"],
        // Old Office binaries and HEIC: the useful signature sits past the start, and refusing on
        // it would reject valid files. Extension and size are the only checks that apply.
        _ => return true,
    };
    firma
        .iter()
        .any(|prefijo| bytes.len() >= prefijo.len() && &bytes[..prefijo.len()] == *prefijo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_extension_se_reconoce_sin_distinguir_mayusculas() {
        assert_eq!(de_extension("PDF"), Some("application/pdf"));
        assert_eq!(de_extension("Docx"), de_extension("docx"));
    }

    #[test]
    fn una_extension_fuera_de_la_lista_no_tiene_mime() {
        // The legacy code answered `application/octet-stream` here and accepted the file.
        assert_eq!(de_extension("exe"), None);
        assert_eq!(de_extension("dll"), None);
        assert_eq!(de_extension(""), None);
    }

    #[test]
    fn los_dos_jpeg_dan_el_mismo_mime() {
        assert_eq!(de_extension("jpg"), de_extension("jpeg"));
    }

    #[test]
    fn un_pdf_de_verdad_coincide_y_un_ejecutable_renombrado_no() {
        assert!(contenido_coincide("pdf", b"%PDF-1.7\n..."));
        assert!(!contenido_coincide("pdf", b"MZ\x90\x00ejecutable"));
    }

    #[test]
    fn los_contenedores_zip_aceptan_la_firma_pk() {
        for ext in ["zip", "docx", "xlsx", "odt", "ods"] {
            assert!(contenido_coincide(ext, b"PK\x03\x04resto"), "{ext}");
            assert!(!contenido_coincide(ext, b"%PDF-1.7"), "{ext}");
        }
    }

    #[test]
    fn un_formato_sin_firma_estable_no_se_rechaza() {
        assert!(contenido_coincide("txt", b"cualquier cosa"));
        assert!(contenido_coincide("csv", b"a,b,c"));
        assert!(contenido_coincide("heic", b"\x00\x00\x00 ftypheic"));
    }

    #[test]
    fn un_archivo_mas_corto_que_la_firma_no_coincide() {
        assert!(!contenido_coincide("pdf", b"%PD"));
        assert!(!contenido_coincide("png", &[]));
    }
}
