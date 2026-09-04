use super::*;


    fn canvas() -> Canvas {
        Canvas::new(
            "prueba",
            theme::page::A4_WIDTH,
            theme::page::A4_HEIGHT,
            theme::page::MARGIN_MOVIMIENTOS,
        )
        .unwrap()
    }

    #[test]
    fn el_ancho_util_descuenta_los_dos_margenes() {
        let c = canvas();
        assert!((c.content_width() - (theme::page::A4_WIDTH - 2.0 * 28.35)).abs() < 0.01);
    }

    #[test]
    fn pedir_mas_espacio_del_que_queda_abre_una_pagina() {
        let c = canvas();
        assert!(!c.ensure_space(100.0));
        assert_eq!(c.page_count(), 1);
        assert!(c.ensure_space(10_000.0));
        assert_eq!(c.page_count(), 2);
        assert_eq!(c.cursor(), theme::page::MARGIN_MOVIMIENTOS);
    }

    #[test]
    fn el_texto_que_no_cabe_se_recorta_con_puntos_suspensivos() {
        let recortado = Canvas::fit(
            "Un concepto larguísimo que no entra",
            10.0,
            FontStyle::Regular,
            40.0,
        );
        assert!(recortado.ends_with('…'), "{recortado}");
        assert!(recortado.chars().count() < 12, "{recortado}");
    }

    #[test]
    fn el_texto_que_cabe_no_se_toca() {
        assert_eq!(
            Canvas::fit("Cable", 10.0, FontStyle::Regular, 200.0),
            "Cable"
        );
    }

    #[test]
    fn un_ancho_ridiculo_devuelve_vacio_en_lugar_de_solo_puntos() {
        assert_eq!(Canvas::fit("Cable", 10.0, FontStyle::Regular, 2.0), "");
    }

    #[test]
    fn el_documento_se_guarda_con_su_pie_en_cada_pagina() {
        let c = canvas();
        c.text_at(&TextSpec::new("Hola", 10.0), c.left(), c.cursor());
        c.new_page();
        let bytes = c
            .finish(|actual, total| {
                Some(TextSpec::new(format!("{actual}/{total}"), 8.0).align(Align::Center))
            })
            .unwrap();
        assert!(bytes.starts_with(b"%PDF"), "no es un PDF");
    }
