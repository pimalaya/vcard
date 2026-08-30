---
cairn: tasks
change: param-quotes-are-delimiters
---

# Tasks

- [x] Add `VcardEscaper::has_param_quoting`, false for 2.1 alone
- [x] Strip a balanced surrounding quote pair in `unescape_param`
- [x] Wrap in a quote pair in `escape_param` when a delimiter needs it
- [x] Replace the tests pinning the old behaviour and cover the new one
- [x] Run the full suite, the corpus and the differential included
