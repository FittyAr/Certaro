//! Paging derives every count from the request and the total, so no caller gets `has_next` wrong
//! on the last page. See `docs/04-dinero-fechas-y-tipos.md` §8.

use certaro_application::{AppError, PageRequest, PagedResult};
use pretty_assertions::assert_eq;

#[test]
fn el_default_es_pagina_uno_de_treinta() {
    let r = PageRequest::default();
    assert_eq!(r.page, 1);
    assert_eq!(r.size, PageRequest::DEFAULT_SIZE);
    assert_eq!(r.size, 30);
    assert_eq!(r.offset(), 0);
    assert_eq!(r.limit(), Some(30));
}

#[test]
fn el_offset_es_cero_en_la_primera_pagina() {
    assert_eq!(PageRequest::new(1, 30).offset(), 0);
    assert_eq!(PageRequest::new(2, 30).offset(), 30);
    assert_eq!(PageRequest::new(4, 10).offset(), 30);
}

#[test]
fn tamano_cero_significa_sin_paginar() {
    let r = PageRequest::new(1, 0);
    assert_eq!(r.limit(), None);

    let result = PagedResult::new(vec![1, 2, 3], 3, r);
    assert_eq!(result.total_pages, 1);
    assert!(!result.has_next);
    assert!(!result.has_previous);
}

#[test]
fn sin_paginar_y_sin_filas_no_hay_paginas() {
    let result: PagedResult<i32> = PagedResult::new(Vec::new(), 0, PageRequest::new(1, 0));
    assert_eq!(result.total_pages, 0);
}

#[test]
fn el_total_de_paginas_redondea_hacia_arriba() {
    let cases = [(0u64, 0u32), (1, 1), (30, 1), (31, 2), (60, 2), (61, 3)];
    for (total, expected) in cases {
        let result: PagedResult<i32> = PagedResult::new(Vec::new(), total, PageRequest::new(1, 30));
        assert_eq!(result.total_pages, expected, "total = {total}");
    }
}

#[test]
fn la_ultima_pagina_no_tiene_siguiente() {
    let last = PagedResult::new(vec![1], 61, PageRequest::new(3, 30));
    assert!(last.has_previous);
    assert!(!last.has_next);

    let middle = PagedResult::new(vec![1], 61, PageRequest::new(2, 30));
    assert!(middle.has_previous);
    assert!(middle.has_next);
}

#[test]
fn solo_se_aceptan_los_cinco_tamanos_de_pagina() {
    for size in PageRequest::ALLOWED_SIZES {
        assert!(
            PageRequest::new(1, size).validate().is_ok(),
            "size = {size}"
        );
    }
    // Nobody gets to ask for a million rows.
    for size in [1u32, 7, 29, 1_000, 1_000_000] {
        let err = PageRequest::new(1, size).validate().expect_err("rejected");
        assert!(matches!(err, AppError::Validation(_)), "size = {size}");
    }
}

#[test]
fn la_pagina_cero_no_existe() {
    let err = PageRequest::new(0, 30).validate().expect_err("rejected");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.iter().any(|f| f.field == "page"));
        }
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[test]
fn map_conserva_todos_los_contadores() {
    let source = PagedResult::new(vec![1, 2, 3], 61, PageRequest::new(2, 30));
    let mapped = source.map(|n| n.to_string());

    assert_eq!(mapped.items, vec!["1", "2", "3"]);
    assert_eq!(mapped.total_count, 61);
    assert_eq!(mapped.total_pages, 3);
    assert_eq!(mapped.page, 2);
    assert!(mapped.has_previous);
    assert!(mapped.has_next);
}

#[test]
fn try_map_propaga_el_primer_error() {
    let source = PagedResult::new(vec![1, 2, 3], 3, PageRequest::default());
    let result: Result<PagedResult<i32>, &str> =
        source.try_map(|n| if n == 2 { Err("boom") } else { Ok(n) });
    assert_eq!(result.err(), Some("boom"));
}
