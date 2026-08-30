//! Sanitising of attachment file names. See `docs/13-servicios-externos-y-archivos.md` §1.3.
//!
//! The name comes from a file the user picked, so it is arbitrary text that is about to become part
//! of a path. Everything that could make it mean something to the filesystem is removed here.

/// Longest name that survives, in characters, extension included.
const MAX: usize = 200;

/// What a name that sanitises down to nothing becomes.
const FALLBACK: &str = "archivo";

/// Device names Windows reserves. `con.txt` opens the console, not a file, and the same name on
/// Linux is harmless — so it is handled everywhere rather than per platform, because the data
/// directory can be synced between machines.
const RESERVADOS: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// A safe file name: no separators, no traversal, no control characters, no reserved device name.
#[must_use]
pub fn sanitize(nombre: &str) -> String {
    // Only the last segment matters: `../../etc/passwd` is the file `passwd`.
    let base = nombre
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(nombre);

    let limpio: String = base
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .collect();

    let colapsado = colapsar_espacios(limpio.trim());
    // Leading dots would make the file hidden and `..` would still read as traversal.
    let sin_puntos = colapsado.trim_start_matches('.').trim_end_matches('.').trim();

    let (raiz, extension) = partir(sin_puntos);
    let raiz = if es_reservado(&raiz) {
        format!("_{raiz}")
    } else {
        raiz
    };

    let nombre = recortar(&raiz, &extension);
    if nombre.is_empty() {
        FALLBACK.to_owned()
    } else {
        nombre
    }
}

/// The extension of an already sanitised name, lowercase and without the dot.
#[must_use]
pub fn extension_de(nombre: &str) -> String {
    partir(nombre).1.to_lowercase()
}

fn colapsar_espacios(texto: &str) -> String {
    let mut out = String::with_capacity(texto.len());
    for c in texto.chars() {
        let espacio = c == ' ';
        if espacio && out.ends_with(' ') {
            continue;
        }
        out.push(if espacio { ' ' } else { c });
    }
    out
}

/// Splits into stem and extension. A name that is only an extension — `.gitignore`, already stripped
/// of its dot by then — has no extension at all.
fn partir(nombre: &str) -> (String, String) {
    match nombre.rsplit_once('.') {
        Some((raiz, ext)) if !raiz.is_empty() && !ext.is_empty() => {
            (raiz.to_owned(), ext.to_owned())
        }
        _ => (nombre.to_owned(), String::new()),
    }
}

fn es_reservado(raiz: &str) -> bool {
    RESERVADOS.contains(&raiz.to_lowercase().as_str())
}

/// Caps the length while keeping the extension: a `.pdf` that loses its extension stops opening.
fn recortar(raiz: &str, extension: &str) -> String {
    if extension.is_empty() {
        return raiz.chars().take(MAX).collect();
    }
    let disponible = MAX.saturating_sub(extension.chars().count() + 1);
    let raiz: String = raiz.chars().take(disponible).collect();
    if raiz.is_empty() {
        // Nothing left for a stem, so the extension alone becomes the name.
        extension.chars().take(MAX).collect()
    } else {
        format!("{raiz}.{extension}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_nombre_normal_no_se_toca() {
        assert_eq!(sanitize("factura_luz.pdf"), "factura_luz.pdf");
        assert_eq!(sanitize("Presupuesto Obra Sur.xlsx"), "Presupuesto Obra Sur.xlsx");
    }

    #[test]
    fn la_travesia_de_directorios_se_queda_en_el_archivo() {
        assert_eq!(sanitize("../../etc/passwd"), "passwd");
        assert_eq!(sanitize("..\\..\\windows\\system32\\cmd.exe"), "cmd.exe");
        assert_eq!(sanitize("/absoluto/factura.pdf"), "factura.pdf");
    }

    #[test]
    fn los_nombres_reservados_de_windows_se_desactivan() {
        assert_eq!(sanitize("con.txt"), "_con.txt");
        assert_eq!(sanitize("COM1.pdf"), "_COM1.pdf");
        // Only the exact device name is reserved: this one is a real file.
        assert_eq!(sanitize("contrato.pdf"), "contrato.pdf");
    }

    #[test]
    fn los_caracteres_invalidos_y_de_control_se_reemplazan() {
        assert_eq!(sanitize("fac*tura?.pdf"), "fac_tura_.pdf");
        assert_eq!(sanitize("nota\u{7}\u{1}.txt"), "nota__.txt");
    }

    #[test]
    fn los_puntos_iniciales_y_finales_no_sobreviven() {
        assert_eq!(sanitize("...oculto.pdf"), "oculto.pdf");
        assert_eq!(sanitize("archivo.pdf..."), "archivo.pdf");
        assert_eq!(sanitize(".."), FALLBACK);
    }

    #[test]
    fn los_espacios_repetidos_se_colapsan() {
        assert_eq!(sanitize("  factura    de   luz.pdf  "), "factura de luz.pdf");
    }

    #[test]
    fn un_nombre_que_queda_vacio_usa_el_respaldo() {
        assert_eq!(sanitize(""), FALLBACK);
        assert_eq!(sanitize("   "), FALLBACK);
    }

    #[test]
    fn el_recorte_conserva_la_extension() {
        let largo = format!("{}.pdf", "a".repeat(500));
        let saneado = sanitize(&largo);
        assert_eq!(saneado.chars().count(), MAX);
        assert!(saneado.ends_with(".pdf"), "{saneado}");
    }

    #[test]
    fn la_extension_se_lee_en_minusculas() {
        assert_eq!(extension_de("Factura.PDF"), "pdf");
        assert_eq!(extension_de("sin_extension"), "");
    }
}
