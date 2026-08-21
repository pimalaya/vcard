---
cairn: log
change: docs-concision
landed: 2026-08-21
---

# Trim the documentation and fix what went stale

A sweep over every documentation surface (the lib.rs header, the module headers, the item docs, and the markdown files) for concision and for statements the code had outgrown. No behaviour, no public API and no capability moved: this is prose only, 46 files and roughly a hundred net lines lighter.

**Concision.** The lib.rs header keeps its six sections but states each in fewer words, and its feature list now names the crate each feature pulls. The module headers that had grown to three or four paragraphs (cst, merge, builder, validate, jcard, jscontact, line, node, cursor, codec, decode, encode) are back to a summary line plus one or two paragraphs. cst.rs dropped its second doctest, which duplicated the builder and validation examples those two modules already carry; the parse-edit-decode one stays.

**Repeated boilerplate.** The twelve decoded-value modules each closed on the same two sentences about being pure unescaped data whose wire name lives on `VcardProp::name`; that is stated once in the parent value.rs header now. The same went for the four `VcardValueNode` mutators that each re-promised to leave their siblings' bytes untouched, now a single sentence on the type, and for the five version-forked lens headers (`GEO`, `KEY`, `LOGO`, `PHOTO`, `SOUND`), which shared an identical five-line paragraph.

**Stale statements.** The value cursor called itself the cursor used by every lens but `N`, when `ADR`, `GENDER` and `CLIENTPIDMAP` have had bespoke cursors for a while. The spec vtable justified itself against a 42-arm match, and the vocabulary holds 48 kinds. vcard.rs promised "the wire names that frame it" and version.rs "its name vocabulary", neither of which those modules have carried since the envelope names moved to the tree. prop.rs and param.rs pointed at a "decode registry" that is a spec lookup and a match. SECURITY.md still supported 0.1.x.

**Markdown.** The two CONTRIBUTING deviations lost a paragraph each without losing an argument, and the unreleased CHANGELOG entry was folded into the summary-plus-paragraph shape changelog-001 asks for, rather than one long line.
