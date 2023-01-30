---
"nodejs-binding": patch
---

Return new `ParticipationEventWithNodes` interface instead of `ParticipationEvent, Node[]`.
This was also changed in the db, old events will not be loaded.
