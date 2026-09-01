//! The mandatory date tests of `docs/17-testing.md` §2.2.

use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::America::Argentina::Buenos_Aires;
use certaro_domain::clock::{Clock, FixedClock};
use certaro_domain::time::{
    civil_to_utc, from_storage, local_to_utc, parse_civil, range_end, range_start, to_storage,
    utc_to_civil,
};
use certaro_domain::RowVersion;
use pretty_assertions::assert_eq;

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

#[test]
fn civil_to_utc_es_medianoche() {
    let dt = civil_to_utc(day(2026, 8, 29));
    assert_eq!(to_storage(dt), "2026-08-29T00:00:00.000Z");
}

#[test]
fn civil_roundtrip() {
    let mut d = day(2024, 1, 1);
    // A leap year plus a few months, so 29 February and every month length are covered.
    for _ in 0..500 {
        assert_eq!(utc_to_civil(civil_to_utc(d)), d);
        d = d.succ_opt().expect("in range");
    }
}

#[test]
fn civil_no_cambia_de_dia_cerca_de_medianoche() {
    // The case that broke the legacy system: an attendance saved at 22:30 local time. Reading it
    // back as a civil date must give the same calendar day, not the next one.
    let local = day(2026, 8, 29).and_hms_opt(22, 30, 0).expect("valid time");
    let instant = local_to_utc(local, &Buenos_Aires);

    // In UTC it is already the 30th…
    assert_eq!(to_storage(instant), "2026-08-30T01:30:00.000Z");
    // …but the civil date the user chose is stored as midnight of the 29th and comes back intact.
    assert_eq!(
        utc_to_civil(civil_to_utc(day(2026, 8, 29))),
        day(2026, 8, 29)
    );
}

#[test]
fn rango_de_dia_incluye_el_ultimo_milisegundo() {
    let d = day(2026, 8, 31);
    assert_eq!(to_storage(range_start(d)), "2026-08-31T00:00:00.000Z");
    assert_eq!(to_storage(range_end(d)), "2026-08-31T23:59:59.999Z");

    let last_moment = Utc
        .with_ymd_and_hms(2026, 8, 31, 23, 59, 59)
        .single()
        .expect("valid instant")
        + chrono::Duration::milliseconds(999);
    assert!(
        last_moment <= range_end(d),
        "the last millisecond is inside"
    );
}

#[test]
fn formato_de_almacenamiento_tiene_24_caracteres() {
    for instant in [
        civil_to_utc(day(2000, 1, 1)),
        range_end(day(2026, 12, 31)),
        Utc.with_ymd_and_hms(2026, 8, 29, 15, 4, 5)
            .single()
            .expect("valid instant"),
    ] {
        assert_eq!(to_storage(instant).len(), 24, "{}", to_storage(instant));
    }
}

#[test]
fn el_orden_lexicografico_coincide_con_el_cronologico() {
    let earlier = to_storage(civil_to_utc(day(2026, 8, 9)));
    let later = to_storage(civil_to_utc(day(2026, 8, 10)));
    // Zero padding is what makes this hold; "2026-8-9" would sort after "2026-8-10".
    assert!(earlier < later);
}

#[test]
fn parse_civil_acepta_la_forma_del_frontend_y_rechaza_el_resto() {
    assert_eq!(parse_civil("2026-08-29").expect("parses"), day(2026, 8, 29));
    assert_eq!(
        parse_civil(" 2026-08-29 ").expect("parses"),
        day(2026, 8, 29)
    );
    for bad in ["29/08/2026", "2026-13-01", "2026-08-29T00:00:00Z", ""] {
        assert!(parse_civil(bad).is_err(), "{bad:?} should be rejected");
    }
}

#[test]
fn from_storage_normaliza_un_offset_a_utc() {
    let dt = from_storage("2026-08-29T22:00:00.000-03:00").expect("parses");
    assert_eq!(to_storage(dt), "2026-08-30T01:00:00.000Z");
}

#[test]
fn el_reloj_fijo_no_avanza() {
    // 21:30 on 29 August in Argentina is already 00:30 on the 30th in UTC.
    let instant = Utc
        .with_ymd_and_hms(2026, 8, 30, 0, 30, 0)
        .single()
        .expect("valid instant");
    let clock = FixedClock(instant);

    assert_eq!(clock.now_utc(), instant);
    assert_eq!(clock.now_utc(), instant);
    // Which is exactly why "today" needs the zone and cannot be derived from `now_utc` alone.
    assert_eq!(utc_to_civil(clock.now_utc()), day(2026, 8, 30));
    assert_eq!(clock.today_civil(&Buenos_Aires), day(2026, 8, 29));
}

#[test]
fn row_version_incrementa_en_big_endian() {
    assert_eq!(RowVersion::INITIAL.as_u64(), 1);
    assert_eq!(RowVersion::INITIAL.to_hex(), "0000000000000001");
    assert_eq!(RowVersion::INITIAL.next().as_u64(), 2);
    assert_eq!(RowVersion::INITIAL.next().to_hex(), "0000000000000002");
}

#[test]
fn row_version_desborda_sin_panic() {
    let max = RowVersion::from_bytes([0xff; 8]);
    assert_eq!(max.next().as_u64(), 0);
}

#[test]
fn row_version_hexadecimal_es_ida_y_vuelta() {
    let v = RowVersion::from_bytes([0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(RowVersion::parse_hex(&v.to_hex()).expect("parses"), v);

    for bad in ["", "00", "zzzzzzzzzzzzzzzz", "000000000000000"] {
        assert!(
            RowVersion::parse_hex(bad).is_err(),
            "{bad:?} should be rejected"
        );
    }
}

#[test]
fn row_version_exige_exactamente_ocho_bytes() {
    assert!(RowVersion::from_slice(&[0; 8]).is_ok());
    assert!(RowVersion::from_slice(&[0; 7]).is_err());
    assert!(RowVersion::from_slice(&[0; 9]).is_err());
}
