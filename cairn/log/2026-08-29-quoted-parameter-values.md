---
cairn: log
change: quoted-parameter-values
landed: 2026-08-29
---

# A quoted parameter value may carry a colon and a semicolon

The line splitter cut the head at the first `:` anywhere and split parameters on every `;`, ignoring RFC 6350 section 3.3, which lets a double-quoted parameter value hold both. The RFC's own section 6.3.1 `ADR` example, which the corpus carries, parsed to one parameter holding `"geo`, no `LABEL`, and an address shifted by one component; `TZ="05:45"` broke the same way. Round-tripping hid it, since every piece is written back verbatim.

Both scans are now quote aware, with a fallback to the quote-blind scan when an unbalanced quote leaves no colon outside quotes, so a junk line still parses rather than failing.

Spec updated: `parsing` (ADDED: a quoted parameter value is opaque).
