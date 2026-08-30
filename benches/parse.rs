//! Parse and serialize benchmarks, comparing like with like by level.
//!
//! `parse_to_content_lines` pits this crate's byte-faithful CST against
//! `ical_vcard` and `vparser`, which are also low-level content-line parsers
//! (no decoding).
//!
//! `parse_to_model` pits our parse plus decode (to the `Vcard` model, not the
//! validated `VcardValid<Vcard>`) against the eager model parsers `calcard`
//! and `vcard_parser`. The `vcard` crate is builder-only, so it never parses.
//!
//! Representations still differ in laziness, ownership and decoding depth, so
//! read these as ballpark rather than a strict ranking.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use vcard::tree::cst::VcardCst;

/// A realistic vCard 4.0 card with a typical property mix.
const CARD: &str = concat!(
    "BEGIN:VCARD\r\n",
    "VERSION:4.0\r\n",
    "FN:Dr. John Q. Public\\, Esq.\r\n",
    "N:Public;John;Quinlan;Dr.;Esq.\r\n",
    "NICKNAME:Johnny,JQ\r\n",
    "ORG:ABC Inc.;North American Division;Marketing\r\n",
    "TITLE:Chief Marketing Officer\r\n",
    "EMAIL;TYPE=work:jqpublic@example.com\r\n",
    "EMAIL;TYPE=home:john@example.net\r\n",
    "TEL;TYPE=\"work,voice\";VALUE=uri:tel:+1-555-555-1212\r\n",
    "TEL;TYPE=\"cell,text\";VALUE=uri:tel:+1-555-555-3434\r\n",
    "ADR;TYPE=work:;;123 Main Street;Any Town;CA;91921-1234;U.S.A.\r\n",
    "URL:https://example.com/jqpublic\r\n",
    "BDAY:19700310\r\n",
    "NOTE:A fairly typical contact card with several properties.\r\n",
    "REV:20080424T195243Z\r\n",
    "END:VCARD\r\n",
);

/// Content-line level: a lazy, byte-faithful split, no value decoding. Compared
/// against `ical_vcard`, which works at the same level.
fn parse_to_content_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_content_lines");

    group.bench_function("vcard-rs: VcardCst::parse", |b| {
        b.iter(|| VcardCst::parse(black_box(CARD)).unwrap())
    });
    group.bench_function("ical_vcard: Parser", |b| {
        b.iter(|| {
            ical_vcard::Parser::new(black_box(CARD).as_bytes())
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        })
    });
    group.bench_function("vparser: Parser", |b| {
        b.iter(|| vparser::Parser::new(black_box(CARD)).collect::<Vec<_>>())
    });

    group.finish();
}

/// Decoded-model level: full parse into a typed model.
///
/// Ours is parse plus decode into `Vcard`, not the validated
/// `VcardValid<Vcard>`, compared against the eager model parsers.
fn parse_to_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_model");

    group.bench_function("vcard-rs: parse + decode", |b| {
        b.iter(|| {
            let cst = VcardCst::parse(black_box(CARD)).unwrap();
            black_box(cst.decode());
        })
    });
    group.bench_function("calcard", |b| {
        b.iter(|| black_box(calcard::vcard::VCard::parse(black_box(CARD))))
    });
    group.bench_function("vcard_parser", |b| {
        b.iter(|| black_box(vcard_parser::parse_vcards(black_box(CARD))))
    });

    group.finish();
}

/// This crate's own decode/encode pipeline.
fn vcard_rs_pipeline(c: &mut Criterion) {
    let cst = VcardCst::parse(CARD).unwrap();
    let card = cst.decode();

    c.bench_function("vcard-rs: decode", |b| {
        b.iter(|| black_box(black_box(&cst).decode()))
    });
    c.bench_function("vcard-rs: to_string (encode)", |b| {
        b.iter(|| black_box(black_box(&card).to_string()))
    });
    c.bench_function("vcard-rs: to_bytes", |b| {
        b.iter(|| black_box(black_box(&cst).to_bytes()))
    });
    c.bench_function("vcard-rs: round-trip (parse + to_bytes)", |b| {
        b.iter(|| VcardCst::parse(black_box(CARD)).unwrap().to_bytes())
    });
}

criterion_group!(
    benches,
    parse_to_content_lines,
    parse_to_model,
    vcard_rs_pipeline
);
criterion_main!(benches);
