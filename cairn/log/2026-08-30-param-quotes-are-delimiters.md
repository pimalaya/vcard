---
cairn: log
change: param-quotes-are-delimiters
landed: 2026-08-30
---

# A parameter's double quotes are the grammar's, not the value's

The decoded model held a parameter exactly as the wire spelled it, its RFC 6350 section 3.3 quotes included. Every consumer read them: `line.param::<GEO>()` on the RFC's own section 6.3.1 address handed back `"geo:12.3457,78.910"`, quotes and all, which no URI parser takes; the jCard and JSContact exports wrote them into a JSON string that has no vCard quoting; and the merge, comparing through the same decoder, read `TZ="America/New_York"` and `TZ=America/New_York` as two values, so a server that re-quoted a parameter reported a change nobody made.

`unescape_param` now strips a balanced surrounding pair before resolving the carets, and `escape_param` wraps its result in one when the text carries a `,`, a `;` or a `:`. `VcardEscaper::has_param_quoting` keeps vCard 2.1 out of it, its grammar having no quoted-string, and RFC 6868 `^'` becomes reachable: a literal double quote is content in 4.0 and encodes as itself.

Byte fidelity is untouched, the quotes living on the syntax leaf that parsing and serialization never read through the codec. The canonical `decode` and `encode` projections change and stay lossless: a value that needs quoting is quoted again, and `TYPE="work"` comes back as `TYPE=work`, the quotes having had nothing to protect.

It breaks a caller that built a parameter with its own quotes: `VcardParam::Geo("\"geo:...\"")` now means a value whose text starts and ends with a quote and goes out as `GEO="^'geo:...^'"`. Nothing is lost, but the card is not the one that caller meant.

Spec updated: `decoded-model` (MODIFIED: a parameter value is encoded by RFC 6868, not by the text escapes).
