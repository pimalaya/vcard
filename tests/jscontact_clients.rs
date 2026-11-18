//! Cards as real clients write them, through JSContact and back.
//!
//! Every fixture of tests/corpus/clients (an iOS contact and group, a Google
//! contact, an Outlook contact, a Thunderbird contact) must come back from its
//! Card line for line once normalized, with every `X-` property and parameter
//! it carries. A 3.0 card goes up to 4.0 first and back down to 3.0 after.

#![cfg(all(feature = "jscontact", feature = "parser"))]

mod common;

use vcard::{tree::cst::VcardCst, vcard::Vcard, version::VcardVersion};

/// Splits `text` on `sep` outside double quotes.
fn split_unquoted(text: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut quoted = false;
    let mut start = 0;

    for (at, char) in text.char_indices() {
        match char {
            '"' => quoted = !quoted,
            _ if char == sep && !quoted => {
                out.push(&text[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }

    out.push(&text[start..]);
    out
}

/// A parameter normalized: its name upper-cased, a `TYPE` or `VALUE` too, the
/// quotes around a value needing none dropped (RFC 6350 3.3).
fn param(param: &str) -> Vec<String> {
    let Some((name, value)) = param.split_once('=') else {
        return vec![param.to_ascii_uppercase()];
    };

    let name = name.to_ascii_uppercase();
    let bare = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .filter(|inner| !inner.contains([':', ';', ',']))
        .unwrap_or(value);

    match name.as_str() {
        "TYPE" => bare
            .split(',')
            .map(|item| format!("TYPE={}", item.to_ascii_uppercase()))
            .collect(),
        "VALUE" => vec![format!("VALUE={}", bare.to_ascii_uppercase())],
        _ => vec![format!("{name}={bare}")],
    }
}

/// The `VALUE` a property means with none stated (RFC 6350 6.2.5, 6.4.1).
const DEFAULT_TYPES: [(&str, &str); 3] = [
    ("BDAY", "VALUE=DATE"),
    ("ANNIVERSARY", "VALUE=DATE"),
    ("TEL", "VALUE=TEXT"),
];

/// The content lines of a card, normalized as a round trip through
/// JSContact may change them, sorted.
///
/// Folds are undone, names upper-cased, `TYPE` lists split one value a
/// parameter and upper-cased, parameters sorted, a `VALUE` stating the
/// default dropped, a date or timestamp spelled basic, a `\:` (no RFC
/// escape) read as `:`, the items of a `CATEGORIES` sorted (keywords are a
/// set), and the conversion's own `PROP-ID` keys left out.
fn lines(vcf: &str) -> Vec<String> {
    let mut lines: Vec<String> = vcf
        .replace("\r\n ", "")
        .replace("\r\n\t", "")
        .split("\r\n")
        .filter(|line| !line.is_empty())
        .map(|line| {
            let colon = split_unquoted(line, ':')[0].len();
            let (head, value) = (&line[..colon], &line[colon + 1..]);
            let mut params: Vec<String> = split_unquoted(head, ';')
                .into_iter()
                .flat_map(param)
                .collect();
            let name = params.remove(0);
            let bare = name.rsplit('.').next().unwrap_or(&name).to_string();
            params.retain(|param| {
                let value = param.to_ascii_uppercase();
                !param.starts_with("PROP-ID=")
                    && !DEFAULT_TYPES.contains(&(bare.as_str(), value.as_str()))
            });
            params.sort_unstable();
            params.dedup();

            let value = value.replace("\\:", ":");
            let value = match bare.as_str() {
                "BDAY" | "ANNIVERSARY" | "REV" => value.replace(['-', ':'], ""),
                "CATEGORIES" => {
                    let mut items: Vec<&str> = value.split(',').collect();
                    items.sort_unstable();
                    items.join(",")
                }
                _ => value,
            };

            match params.is_empty() {
                true => format!("{name}:{value}"),
                false => format!("{name};{}:{value}", params.join(";")),
            }
        })
        .collect();

    lines.sort_unstable();
    lines
}

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/corpus/clients/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("fixture")
}

/// What a fixture lost or gained through JSContact, if anything.
fn round_trip(name: &str) -> Option<String> {
    let vcf = fixture(name);
    let cst = VcardCst::parse(&vcf).expect("a card");
    let card = cst.decode();
    let version = card.version;
    let up = card.to_version(VcardVersion::V4_0);
    let json = up.to_jscontact();
    let back = Vcard::from_jscontact(&json).expect("a Card");
    let down = back.to_version(version);
    let text = VcardCst::from(down).to_string();

    let expected = lines(&vcf);
    let actual = lines(&text);
    let missing: Vec<_> = expected.iter().filter(|l| !actual.contains(l)).collect();
    let extra: Vec<_> = actual.iter().filter(|l| !expected.contains(l)).collect();

    if !missing.is_empty() || !extra.is_empty() {
        return Some(format!("{name}\nmissing: {missing:#?}\nextra: {extra:#?}"));
    }

    let again = Vcard::from_jscontact(&json).expect("a Card").to_jscontact();

    if again != json {
        return Some(format!("{name}: a second pass changed the Card"));
    }

    None
}

const FIXTURES: [&str; 5] = [
    "ios-contact.vcf",
    "ios-group.vcf",
    "google-contact.vcf",
    "outlook-contact.vcf",
    "thunderbird-contact.vcf",
];

#[test]
fn every_client_card_round_trips_through_jscontact() {
    common::each_fixture("clients", FIXTURES.len(), |_, _| {});

    let failures: Vec<String> = FIXTURES
        .iter()
        .filter_map(|name| round_trip(name))
        .collect();

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
