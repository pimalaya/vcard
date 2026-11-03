use std::{fs, hint::black_box, path::Path};
use vcard::tree::cst::VcardCst;

fn walk(p: &Path, out: &mut Vec<Vec<u8>>) {
    let mut es: Vec<_> = fs::read_dir(p).unwrap().flatten().map(|e| e.path()).collect();
    es.sort();
    for e in es {
        if e.is_dir() { walk(&e, out) } else if e.extension().is_some_and(|x| x == "vcf") { out.push(fs::read(e).unwrap()) }
    }
}

fn main() {
    let _ = simd_cpu::SimdCpu::install_detected();
    let mut a = std::env::args().skip(1);
    let work = a.next().unwrap();
    let n: usize = a.next().unwrap().parse().unwrap();
    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/vcard/tests/corpus"), &mut corpus);
    if let Some(short) = work.strip_prefix("card-") {
        corpus = vec![CARD.as_bytes().to_vec(); 300];
        return run(short, n, &corpus);
    }
    run(&work, n, &corpus);
}

fn run(work: &str, n: usize, corpus: &[Vec<u8>]) {
    if work == "one" {
        for _ in 0..n {
            for f in corpus {
                let _ = black_box(VcardCst::parse(black_box(f.as_slice())));
            }
        }
        return;
    }
    let csts = || corpus.iter().flat_map(|f| VcardCst::parse_many(f.as_slice()).flatten()).collect::<Vec<_>>();
    if work == "dec" || work == "tob" {
        let csts = csts();
        for _ in 0..n {
            for cst in &csts {
                if work == "dec" { black_box(black_box(cst).decode()); } else { black_box(black_box(cst).to_bytes()); }
            }
        }
        return;
    }
    if work == "encode" {
        let csts = csts();
        let cards: Vec<_> = csts.iter().map(|c| c.decode()).collect();
        for _ in 0..n {
            for card in &cards {
                black_box(black_box(card).to_string());
            }
        }
        return;
    }
    for _ in 0..n {
        for f in corpus {
            for card in VcardCst::parse_many(black_box(f.as_slice())) {
                let Ok(card) = card else { continue };
                match work {
                    "parse" => { black_box(&card); }
                    "decode" => { black_box(card.decode()); }
                    "bytes" => { black_box(card.to_bytes()); }
                    _ => panic!(),
                }
            }
        }
    }
}

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
