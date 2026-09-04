use super::*;
use super::super::canvas::Canvas;
use super::super::theme;

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
