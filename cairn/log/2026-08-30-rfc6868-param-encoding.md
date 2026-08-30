---
cairn: log
change: rfc6868-param-encoding
date: 2026-08-30
---

# A parameter is not a text value

The parameter codec had been pointed at the text unescaper, and its own doc comment said so: the RFC 6350 3.4 escapes were "the default used wherever the escaping mode is not version-specific (parameters, the version-blind lens path)". Section 3.3 gives a parameter value no escapes at all, which is the whole reason RFC 6868 exists, so the crate was wrong twice at once. A backslash a parameter legitimately carried was eaten on the way in and could not be written back, and a real `^n` or `^'` from a conforming producer reached the caller with its encoding showing.

Both halves are now RFC 6868. `unescape_param` resolves `^n`, `^^` and `^'` and leaves every other caret sequence exactly as written, which section 3.1 requires rather than merely permits, and `escape_param` writes them back. Neither touches a backslash.

Two decisions shaped the rest.

The rules are keyed on the version. RFC 6868 updates RFC 6350 and nothing earlier, so a 2.1 or 3.0 caret is a literal caret and resolving it would corrupt the value. That is what forced `VcardEscaper::Modern` apart: it stood for 3.0 and 4.0 at once, which is fine for value escaping, where the two are identical, and useless for a rule only 4.0 has. `V3_0` and `V4_0` say which, `has_param_encoding` answers the question, and `V4_0` is the default `Modern` was. It also forced the seam the parameter side had never had: `VcardParamNode` now carries an `escaper`, stamped beside the value nodes once `VERSION` is known, and `VcardParam::encode` and `VcardParamLens::encode` take the target mode the way `VcardProp::encode` and the `VcardCodec` trait already did. Twelve lens modules moved with it.

The encoder leaves a quoted value's own delimiters alone. The decoded model holds a parameter exactly as the wire spelled it, the surrounding double quotes included, so encoding the pair would rewrite every quoted URI as `^'...^'` and no `GEO` would survive its own round trip. A quoted value is encoded inside its quotes instead, which keeps the RFC's motivating case working: a `LABEL` carrying a quote mid-value still writes `^'`.

The transition is invisible, as Postel asks. A parameter with neither a caret nor a backslash means the same under both readings, and that is every parameter of every fixture: all 146 corpus fixtures classify exactly as they did, the RFC and GitHub-case suites are unchanged, and so is the calcard cross-check.

A second defect came out of the same reading. The merge compared parameters in their decoded form, and a single-valued parameter decodes its first value alone, so `LABEL=Ada,Lovelace` against `LABEL=Ada,Byron` compared equal: no action was reported, and the right side's edit was dropped without a word. `diff_params` now carries the raw nodes beside the decoded parameters and compares through `param_eq`, value by value, exactly as `value_eq` already did for values, falling back to raw bytes where two cards of different versions share no decoding. `prop_eq` compares parameters the same way, since a mis-match at the equality rung loses a change just as quietly. `right_param_node` reads the right card's node by name and ordinal, which is how the action was raised, rather than searching for a decoded parameter that is not a key. The old decoded comparison survives under the name it always deserved, `same_param`, where it belongs: deciding whether two reported actions are the same change, a list parameter compared as a set.

Capabilities moved: decoded-model, merge.
