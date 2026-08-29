#![cfg(feature = "parser")]
//! Round-trip robustness sweep over vCard fixtures harvested from popular
//! open-source vCard libraries on GitHub, run through the one version-agnostic
//! parser. Each source repo has its own corpus directory under tests/corpus/
//! (with its own ATTRIBUTION.md), so provenance and licensing stay per project.

mod common;

#[test]
fn parses_and_round_trips() {
    // One entry per source repo: its corpus directory and its fixture count.
    let projects = [
        ("emersion", 5),
        ("jeroen", 6),
        ("mixerp", 2),
        ("nuovo", 3),
        ("sabre", 2),
        ("vcardigan", 2),
    ];

    for (project, count) in projects {
        common::each_fixture(project, count, common::round_trips);
    }
}
