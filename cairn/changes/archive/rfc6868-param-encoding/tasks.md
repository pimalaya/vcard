---
cairn: tasks
change: rfc6868-param-encoding
---

- [x] Add an RFC 6868 decoder: `^n` to newline, `^^` to caret, `^'` to double quote, a caret before anything else kept verbatim
- [x] Add the inverse encoder, and point the parameter codec at both instead of the text unescaper
- [x] Stop applying the RFC 6350 3.4 text escapes to parameter values
- [x] Split `VcardEscaper::Modern` into `V3_0` and `V4_0`, RFC 6868 updating RFC 6350 alone
- [x] Carry the escaper on a parameter node, stamped once `VERSION` is known
- [x] Prove a parameter round trips byte for byte, that a lone caret is untouched, and that a 3.0 caret stays literal
- [x] Re-run the golden corpus and account for every fixture that moves (none did)
- [x] Compare parameters on their raw nodes in the merge, a decoded one reading its first value alone
- [x] Keep the twins aligned with ical-rs (RFC 5545 3.2)
