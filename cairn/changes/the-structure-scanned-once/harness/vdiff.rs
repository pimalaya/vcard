//! Differential run: the working tree (`vcard`) against the baseline copy
//! (`vcard_old`), over the corpus and rounds of random edits of it.
//!
//! Run as `vdiff <rounds> [first-round]`.

use std::{fs, panic, path::Path};

const SALTS: &[&str] = &[
    ";", ",", ":", "\\", "\n", "^n^'^^", "\"", "é€", "\r\n", "=", " \t",
];

const TOKENS: &[&[u8]] = &[
    b"\r",
    b"\n",
    b" ",
    b"\t",
    b":",
    b";",
    b",",
    b"\"",
    b"=",
    b"\\",
    b"^",
    b"ENCODING=QUOTED-PRINTABLE",
    b"CHARSET=",
    b"BEGIN:VCARD",
    b"END:VCARD",
    b"VERSION:2.1",
    b"VERSION:3.0",
    b"VERSION:4.0",
    b"item1.",
    b"\xC3\xA9",
    b"\xFF",
    b"\x80",
    b"\xE2\x82",
    b"\r\n ",
    b"\r\n\t",
    b"=\r\n",
    b"\\n",
    b"\\N",
    b"\\,",
    b"\\;",
    b"\\\\",
    b"^n",
    b"^'",
    b"^^",
    b";VALUE=uri",
    b";VALUE=text",
    b";TYPE=\"a,b\"",
    b";TYPE=a,b",
    b";PID=1,2",
    b";X-P=\"a;b:c\"",
    b"QUOTED-PRINTABLE",
    b";CHARSET=ISO-8859-1",
    b";ENCODING=b",
    b"\r\n\r\n",
    b"\r\nADR;TYPE=work:;;1 St;Town;;;\r\n",
    b"\r\nADR:a,b;c\\,d;e\\;f;g;h;i;j;k;l;m;n;o;p;q;r;s;t\r\n",
    b"\r\nN:Doe;J;;;\r\n",
    b"\r\nN:a,b;c\\,d;e\\;f\r\n",
    b"\r\nGEO:1.0;2.0\r\n",
    b"\r\nGEO:1.0,2.0\r\n",
    b"\r\nGENDER:M;x\r\n",
    b"\r\nX-FOO;X-P=a,b:v\\,w;x\r\n",
    b"\r\nCLIENTPIDMAP:1;urn:x\r\n",
    b"\r\nORG:a;b\\;c\r\n",
    b"\r\nNICKNAME:a,b\\,c\r\n",
    b"\r\nCATEGORIES:a,b\r\n",
    b"\r\nPHOTO;ENCODING=b:QUJD\r\n",
    b"\r\nPHOTO;ENCODING=BASE64;TYPE=JPEG:QUJD\r\n",
    b"\r\nTEL;VALUE=uri:tel:1\r\n",
    b"\r\nBDAY:2000\r\n",
    b"\r\nREV:20200101T000000Z\r\n",
    b"\r\nTZ:-05:00\r\n",
    b"\r\nLANG:fr\r\n",
    b"\r\nNOTE;ENCODING=QUOTED-PRINTABLE:a=3Db=\r\nc\r\n",
    b"\r\nVERSION:2.1\r\n",
    b"\r\nVERSION:3.0\r\n",
    b"\r\nFN;LANGUAGE=en;X-A=a^nb^^c^'d:x\r\n",
];

macro_rules! suite {
    ($m:ident, $c:ident) => {
        mod $m {
            use std::borrow::Cow;

            use $c::{
                param::VcardParam,
                prop::VcardPropName,
                tree::{
                    codec::mode::VcardEscaper, cst::VcardCst, value::cursor::VcardValueCursor,
                },
                value::{VcardValue, binary::VcardBinary},
                vcard::Vcard,
            };

            use crate::SALTS;

            const ESCAPERS: [VcardEscaper; 3] =
                [VcardEscaper::V2_1, VcardEscaper::V3_0, VcardEscaper::V4_0];

            fn s(v: &mut Cow<'_, str>, salt: &str, mode: u32) {
                let mut o = String::new();
                if mode & 1 != 0 {
                    o.push_str(salt)
                }
                o.push_str(v);
                if mode & 2 != 0 {
                    o.push_str(salt)
                }
                *v = Cow::Owned(o);
            }

            fn l(v: &mut Vec<Cow<'_, str>>, salt: &str, mode: u32) {
                if v.is_empty() {
                    v.push(Cow::Borrowed(""))
                }
                for x in v.iter_mut() {
                    s(x, salt, mode)
                }
            }

            fn salt(card: &mut Vcard<'_>, salt: &str, mode: u32) {
                for (i, p) in card.properties.iter_mut().enumerate() {
                    if i % 4 == 0 && p.group.is_none() {
                        p.group = Some(Cow::Borrowed("g"));
                    }
                    if let Some(g) = &mut p.group {
                        s(g, salt, mode)
                    }
                    if let VcardPropName::Unknown(n) = &mut p.name {
                        s(n, salt, mode)
                    }
                    for q in p.params.iter_mut() {
                        match q {
                            VcardParam::Pid(v) | VcardParam::SortAs(v) | VcardParam::Type(v) => {
                                l(v, salt, mode)
                            }
                            VcardParam::Unknown { name, values } => {
                                s(name, salt, mode);
                                l(values, salt, mode)
                            }
                            VcardParam::AltId(v)
                            | VcardParam::Author(v)
                            | VcardParam::AuthorName(v)
                            | VcardParam::CalScale(v)
                            | VcardParam::Charset(v)
                            | VcardParam::Created(v)
                            | VcardParam::Derived(v)
                            | VcardParam::Encoding(v)
                            | VcardParam::Geo(v)
                            | VcardParam::Jsptr(v)
                            | VcardParam::Label(v)
                            | VcardParam::Language(v)
                            | VcardParam::MediaType(v)
                            | VcardParam::Phonetic(v)
                            | VcardParam::Pref(v)
                            | VcardParam::PropId(v)
                            | VcardParam::Script(v)
                            | VcardParam::ServiceType(v)
                            | VcardParam::Tz(v)
                            | VcardParam::Username(v)
                            | VcardParam::Value(v) => s(v, salt, mode),
                        }
                    }
                    match &mut p.value {
                        VcardValue::Adr(a) => {
                            for v in [
                                &mut a.po_box,
                                &mut a.extended,
                                &mut a.street,
                                &mut a.locality,
                                &mut a.region,
                                &mut a.postal_code,
                                &mut a.country,
                                &mut a.room,
                                &mut a.apartment,
                                &mut a.floor,
                                &mut a.street_number,
                                &mut a.street_name,
                                &mut a.building,
                                &mut a.block,
                                &mut a.subdistrict,
                                &mut a.district,
                                &mut a.landmark,
                                &mut a.direction,
                            ] {
                                l(v, salt, mode)
                            }
                        }
                        VcardValue::Binary(VcardBinary::Uri(v) | VcardBinary::Base64(v)) => {
                            s(v, salt, mode)
                        }
                        VcardValue::ClientPidMap(c) => {
                            s(&mut c.id, salt, mode);
                            s(&mut c.uri, salt, mode)
                        }
                        VcardValue::DateAndOrTime(v) => s(&mut v.0, salt, mode),
                        VcardValue::Gender(g) => {
                            s(&mut g.sex, salt, mode);
                            s(&mut g.identity, salt, mode)
                        }
                        VcardValue::Geo(g) => {
                            s(&mut g.latitude, salt, mode);
                            s(&mut g.longitude, salt, mode)
                        }
                        VcardValue::LanguageTag(v) => s(&mut v.0, salt, mode),
                        VcardValue::N(n) => {
                            for v in [
                                &mut n.family,
                                &mut n.given,
                                &mut n.additional,
                                &mut n.prefixes,
                                &mut n.suffixes,
                            ] {
                                l(v, salt, mode)
                            }
                        }
                        VcardValue::Org(o) => l(&mut o.0, salt, mode),
                        VcardValue::Text(t) => s(&mut t.0, salt, mode),
                        VcardValue::TextList(t) => l(&mut t.0, salt, mode),
                        VcardValue::Timestamp(t) => s(&mut t.0, salt, mode),
                        VcardValue::Uri(u) => s(&mut u.0, salt, mode),
                        VcardValue::UtcOffset(u) => s(&mut u.0, salt, mode),
                        VcardValue::Unknown(u) => {
                            if u.components.is_empty() {
                                u.components.push(Vec::new())
                            }
                            for c in u.components.iter_mut() {
                                l(c, salt, mode)
                            }
                        }
                    }
                }
            }

            fn model(log: &mut Vec<String>, card: &Vcard<'_>) {
                let direct = card.to_string();
                let cst = card.encode();
                log.push(format!("{cst:?}"));
                log.push(format!("{:?}", cst.to_bytes()));
                log.push(format!("encode-vs-direct {}", cst.to_string() == direct));
                log.push(direct);
                for p in &card.properties {
                    for e in ESCAPERS {
                        let line = p.encode(e);
                        log.push(format!("{line:?}"));
                        log.push(line.to_string());
                    }
                }
            }

            fn reads(log: &mut Vec<String>, cst: &mut VcardCst<'_>) {
                let lines = cst
                    .begin
                    .iter_mut()
                    .chain(cst.props.iter_mut())
                    .chain(cst.end.iter_mut());
                for line in lines {
                    let c = VcardValueCursor { line };
                    log.push(format!("{:?}", c.text()));
                    log.push(format!("{:?}", c.bytes()));
                    log.push(format!("{:?}", c.list()));
                    for i in [0, 1, 2, 7] {
                        log.push(format!("{:?}", c.component(i)));
                    }
                    log.push(format!("{:?}", c.quoted_printable()));
                    log.push(format!("{:?}", c.charset()));
                }
            }

            fn card(log: &mut Vec<String>, cst: &VcardCst<'_>, salts: bool) {
                log.push(format!("{:?}", cst.to_bytes()));
                log.push(cst.to_string());
                log.push(format!("{:?}", cst.version()));
                let card = cst.decode();
                log.push(format!("{card:?}"));
                model(log, &card);
                if salts {
                    for salt_ in SALTS {
                        for mode in 1..=3 {
                            let mut c = card.clone();
                            salt(&mut c, salt_, mode);
                            model(log, &c);
                        }
                    }
                }
                let mut c = cst.clone();
                reads(log, &mut c);
            }

            /// Every observation of one input, in order.
            pub fn run(input: &[u8], salts: bool) -> Vec<String> {
                let mut log = Vec::new();
                let one = VcardCst::parse(input);
                log.push(format!("{one:?}"));
                let many: Vec<_> = VcardCst::parse_many(input).collect();
                log.push(format!("{many:?}"));
                for cst in one.iter().chain(many.iter().flatten()) {
                    card(&mut log, cst, salts);
                    let mut edited = cst.clone();
                    for line in edited.props.iter_mut() {
                        VcardValueCursor { line }.set_component(2, &["x,y", "z;"]);
                    }
                    card(&mut log, &edited, salts);
                }
                log
            }

            /// How many cards `parse_many` takes.
            pub fn cards(input: &[u8]) -> usize {
                VcardCst::parse_many(input).flatten().count()
            }
        }
    };
}

suite!(new, vcard);
suite!(old, vcard_old);

fn walk(p: &Path, out: &mut Vec<Vec<u8>>) {
    let mut es: Vec<_> = fs::read_dir(p)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    es.sort();
    for e in es {
        if e.is_dir() {
            walk(&e, out)
        } else if e.extension().is_some_and(|x| x == "vcf") {
            out.push(fs::read(e).unwrap())
        }
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn mutate(file: &[u8], rng: &mut Rng) -> Vec<u8> {
    let mut b = if file.len() > 4096 {
        // NOTE: Most windows start at a card, so the edits land in cards
        // that parse rather than in a headless tail.
        let mut at = rng.below(file.len() - 4096);
        if rng.below(4) != 0
            && let Some(begin) = file[..=at]
                .windows(11)
                .rposition(|w| w == b"BEGIN:VCARD")
        {
            at = begin;
        }
        file[at..(at + 4096).min(file.len())].to_vec()
    } else {
        file.to_vec()
    };
    for _ in 0..1 + rng.below(3) {
        let at = rng.below(b.len() + 1);
        match rng.below(5) {
            0 if !b.is_empty() => {
                b.remove(at.min(b.len() - 1));
            }
            1 if !b.is_empty() => {
                let at = at.min(b.len() - 1);
                let t = TOKENS[rng.below(TOKENS.len())];
                b.splice(at..at + 1, t.iter().copied());
            }
            2 if b.len() > 2 => {
                let from = rng.below(b.len() - 1);
                let len = 1 + rng.below(64.min(b.len() - from));
                let chunk = b[from..from + len].to_vec();
                b.splice(at..at, chunk);
            }
            _ => {
                let t = TOKENS[rng.below(TOKENS.len())];
                b.splice(at..at, t.iter().copied());
            }
        }
    }
    b
}

fn quiet<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    panic::catch_unwind(panic::AssertUnwindSafe(f)).map_err(|e| {
        e.downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

fn check(input: &[u8], salts: bool, tag: &str) {
    let a = quiet(|| new::run(input, salts));
    let b = quiet(|| old::run(input, salts));
    let same = match (&a, &b) {
        (Ok(a), Ok(b)) => a == b,
        (Err(a), Err(b)) => a == b,
        _ => false,
    };
    if same {
        return;
    }
    let path = format!("fail-{tag}.vcf");
    fs::write(&path, input).unwrap();
    match (a, b) {
        (Ok(a), Ok(b)) => {
            let i = a.iter().zip(&b).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()));
            eprintln!("lengths {} {}", a.len(), b.len());
            let (x, y) = (a.get(i).map_or("", |s| s), b.get(i).map_or("", |s| s));
            let at = x
                .bytes()
                .zip(y.bytes())
                .position(|(p, q)| p != q)
                .unwrap_or(x.len().min(y.len()));
            let window = |s: &str| -> String {
                s.bytes().skip(at.saturating_sub(300)).take(900).map(char::from).collect()
            };
            eprintln!("observation {i}, first difference at byte {at}");
            eprintln!("new: {:?}", window(x));
            eprintln!("old: {:?}", window(y));
        }
        (a, b) => {
            eprintln!("new: {:?}", a.err());
            eprintln!("old: {:?}", b.err());
        }
    }
    panic!("mismatch on {tag}, input saved to {path}");
}

fn main() {
    let _ = simd_cpu::SimdCpu::install_detected();
    panic::set_hook(Box::new(|_| {}));
    let mut args = std::env::args().skip(1);
    let rounds: usize = args.next().map_or(600, |a| a.parse().unwrap());
    let first: usize = args.next().map_or(0, |a| a.parse().unwrap());
    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/vcard/tests/corpus"), &mut corpus);

    let (mut inputs, mut cards) = (0, 0);

    if first == 0 {
        for (i, file) in corpus.iter().enumerate() {
            check(file, true, &format!("corpus-{i}"));
            inputs += 1;
            cards += new::cards(file);
        }
        eprintln!("corpus: {inputs} files, {cards} cards");
    }

    for round in first.max(1)..=rounds {
        for (i, file) in corpus.iter().enumerate() {
            let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ ((round as u64) << 20) ^ i as u64);
            let input = mutate(file, &mut rng);
            check(&input, true, &format!("r{round}-f{i}"));
            inputs += 1;
            cards += new::cards(&input);
        }
        if round % 50 == 0 {
            eprintln!("round {round}: {inputs} inputs, {cards} cards");
        }
    }

    eprintln!("all agree: {inputs} inputs, {cards} cards");
}
