#![cfg(feature = "parser")]
//! # Folding
//!
//! An edited or built 3.0 or 4.0 content line folds at 75 octets (RFC 6350
//! 3.2), keeping the folds before its edit; a 2.1 or quoted-printable line
//! never folds, and an unedited line keeps the shape it arrived in.

use vcard::{
    prop::{VcardProp, VcardPropKind, note::NOTE},
    tree::{cst::VcardCst, value::node::VcardValueNode},
};

/// A card of `version` holding `lines`, each ended by `eol`.
fn card(version: &str, lines: &[&str], eol: &str) -> String {
    let mut out = format!("BEGIN:VCARD{eol}VERSION:{version}{eol}");

    for line in lines {
        out.push_str(line);
        out.push_str(eol);
    }

    out.push_str(&format!("END:VCARD{eol}"));
    out
}

/// `logical` folded every `every` octets, `wsp` opening each continuation.
fn folded(logical: &str, every: usize, wsp: char) -> String {
    let mut out = String::new();
    let mut rest = logical;
    let mut room = every;

    while rest.len() > room {
        let (head, tail) = rest.split_at(room);
        out.push_str(head);
        out.push_str("\r\n");
        out.push(wsp);
        rest = tail;
        room = every - 1;
    }

    out.push_str(rest);
    out
}

/// The physical lines of the card's NOTE, line endings excluded.
fn note_lines(card: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut inside = false;

    for line in card.split(|&b| b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);

        if line.starts_with(b"NOTE") {
            inside = true;
        } else if inside && !line.starts_with(b" ") && !line.starts_with(b"\t") {
            break;
        }

        if inside {
            lines.push(line.to_vec());
        }
    }

    lines
}

/// The card's NOTE value, read back.
fn reread(bytes: &[u8]) -> String {
    let mut card = VcardCst::parse(bytes).expect("the output reparses");
    card.prop_mut::<NOTE>().expect("a note").text().into_owned()
}

/// Asserts every physical line of the NOTE is at most 75 octets and valid
/// UTF-8 on its own.
fn assert_folded(bytes: &[u8]) {
    for line in note_lines(bytes) {
        assert!(line.len() <= 75, "{} octets: {line:?}", line.len());
        assert!(str::from_utf8(&line).is_ok(), "split sequence: {line:?}");
    }
}

/// Words a to z repeated: 300 octets with no character a value escapes.
fn words(octets: usize) -> String {
    let mut text = String::new();
    let mut letter = b'a';

    while text.len() < octets {
        text.push_str(&String::from(letter as char).repeat(5));
        text.push(' ');
        letter = if letter == b'z' { b'a' } else { letter + 1 };
    }

    text.truncate(octets);
    text
}

#[test]
fn a_long_note_edited_at_its_end_keeps_the_folds_before_the_edit() {
    let value = words(300);
    let input = card(
        "4.0",
        &[&folded(&format!("NOTE:{value}"), 74, '\t')],
        "\r\n",
    );

    let edited = format!("{}a much longer closing word", &value[..290]);
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().set_text(&edited);
    let output = card.to_bytes();

    let before = note_lines(input.as_bytes());
    let after = note_lines(&output);
    let kept = before.len() - 1;

    assert_eq!(after[..kept], before[..kept], "the lines before the edit");
    assert!(after[kept..].iter().skip(1).all(|line| line[0] == b'\t'));
    assert_folded(&output);
    assert_eq!(reread(&output), edited);
}

#[test]
fn an_edit_in_the_first_physical_line_refolds_the_whole_line() {
    let value = words(200);
    let input = card("4.0", &[&folded(&format!("NOTE:{value}"), 60, ' ')], "\r\n");

    let edited = format!("x{value}");
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().set_text(&edited);
    let output = card.to_bytes();

    assert_eq!(note_lines(&output)[0].len(), 75);
    assert_folded(&output);
    assert_eq!(reread(&output), edited);
}

#[test]
fn a_shrinking_edit_keeps_the_folds_before_it() {
    let value = words(300);
    let input = card("4.0", &[&folded(&format!("NOTE:{value}"), 75, ' ')], "\r\n");

    let edited = value[..250].to_string();
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().set_text(&edited);
    let output = card.to_bytes();

    let before = note_lines(input.as_bytes());
    assert_eq!(note_lines(&output)[..3], before[..3]);
    assert_folded(&output);
    assert_eq!(reread(&output), edited);
}

#[test]
fn a_list_item_edit_keeps_the_folds_before_it() {
    let value = (0..20)
        .map(|i| format!("category{i:02}"))
        .collect::<Vec<_>>();
    let line = format!("CATEGORIES:{}", value.join(","));
    let input = card("4.0", &[&folded(&line, 75, ' ')], "\r\n");

    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    let mut cursor = card
        .prop_mut::<vcard::prop::categories::CATEGORIES>()
        .unwrap();
    cursor.list_mut().set(19, "a much longer last category");
    let output = card.to_bytes();

    let text = String::from_utf8(output).unwrap();
    let first = folded(&line, 75, ' ');
    let first = first.split("\r\n").next().unwrap();
    assert!(text.contains(&format!("{first}\r\n")), "{text}");
    assert!(text.lines().all(|line| line.len() <= 75), "{text}");
}

#[test]
fn a_built_long_line_is_folded() {
    let value = words(200);
    let input = card("4.0", &["FN:Jane"], "\r\n");
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.push(VcardProp::text(VcardPropKind::Note, vec![], value.as_str()));
    let output = card.to_bytes();

    let lines = note_lines(&output);
    assert_eq!(lines[0].len(), 75);
    assert!(lines[1..].iter().all(|line| line[0] == b' '));
    assert_folded(&output);
    assert_eq!(reread(&output), value);
}

#[test]
fn a_fold_never_splits_a_multi_octet_character() {
    let value = format!("{}é{}", "a".repeat(69), "b".repeat(20));
    let input = card("4.0", &["FN:Jane"], "\r\n");
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.push(VcardProp::text(VcardPropKind::Note, vec![], value.as_str()));
    let output = card.to_bytes();

    assert_eq!(note_lines(&output)[0].len(), 74);
    assert_folded(&output);
    assert_eq!(reread(&output), value);
}

#[test]
fn an_edit_keeping_the_length_never_splits_a_multi_octet_character() {
    let input = card(
        "4.0",
        &[&format!("NOTE:{}é\r\n bc", "a".repeat(69))],
        "\r\n",
    );

    let edited = format!("{}éc", "a".repeat(70));
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().set_text(&edited);
    let output = card.to_bytes();

    assert_folded(&output);
    assert_eq!(reread(&output), edited);
}

#[test]
fn a_value_replaced_through_the_tree_is_folded() {
    let value = words(200);
    let input = card("4.0", &["NOTE:short\r\n note"], "\r\n");

    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().line.value = VcardValueNode::parse(value.as_bytes());
    let output = card.to_bytes();

    assert_folded(&output);
    assert_eq!(reread(&output), value);
}

#[test]
fn a_line_ending_on_a_bare_lf_folds_with_a_bare_lf() {
    let value = words(100);
    let input = card("4.0", &["NOTE:short"], "\n");
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().set_text(&value);
    let output = card.to_bytes();

    let text = String::from_utf8(output.clone()).unwrap();
    assert!(text.contains("\n ") && !text.contains('\r'), "{text:?}");
    assert_folded(&output);
    assert_eq!(reread(&output), value);
}

#[test]
fn an_unedited_long_line_keeps_its_shape() {
    let input = card("4.0", &[&format!("NOTE:{}", words(200))], "\r\n");
    let card = VcardCst::parse(input.as_bytes()).unwrap();

    assert_eq!(card.to_bytes(), input.as_bytes());
}

#[test]
fn a_quoted_printable_line_is_never_folded() {
    let input = card("4.0", &["NOTE;ENCODING=QUOTED-PRINTABLE:short"], "\r\n");
    let value = "=41".repeat(40);

    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().set_bytes(&value);
    let output = card.to_bytes();

    assert_eq!(note_lines(&output).len(), 1);
}

#[test]
fn a_2_1_line_is_never_folded() {
    let value = words(200);
    let input = card(
        "2.1",
        &[&format!("NOTE:{}\r\n {}", &value[..60], &value[60..])],
        "\r\n",
    );

    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>()
        .unwrap()
        .set_text(format!("x{value}"));
    card.push(VcardProp::text(VcardPropKind::Note, vec![], value.as_str()));
    let output = String::from_utf8(card.to_bytes()).unwrap();

    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("NOTE"))
            .count(),
        2
    );
    assert!(
        !output.lines().any(|line| line.starts_with(' ')),
        "{output}"
    );
}

#[test]
fn an_unedited_fold_before_a_continuation_byte_keeps_its_place() {
    let mut input = b"BEGIN:VCARD\r\nVERSION:3.0\r\nNOTE:x\r\n ".to_vec();
    input.extend_from_slice(b"\x85y\r\nEND:VCARD\r\n");
    let card = VcardCst::parse(&input).unwrap();

    assert_eq!(card.to_bytes(), input);
}

#[test]
fn a_decoded_card_writes_the_folds_its_encoding_writes() {
    let input = card("4.0", &["FN:Jane", "NOTE:short"], "\r\n");
    let mut card = VcardCst::parse(input.as_bytes()).unwrap();
    card.prop_mut::<NOTE>().unwrap().set_text(words(200));
    card.push(VcardProp::text(
        VcardPropKind::Note,
        vec![],
        format!("é{}", words(150)),
    ));

    let decoded = card.decode();
    let text = decoded.to_string();

    assert_eq!(text.as_bytes(), decoded.encode().to_bytes());
    assert_eq!(card.to_string().as_bytes(), card.to_bytes());
    assert_folded(text.as_bytes());
}
