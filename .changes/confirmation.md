---
"nodejs-binding": patch
---

Don't retry pruned messages forever, inputs are checked if they're spent so the status can be updated even if the messages got pruned already.
