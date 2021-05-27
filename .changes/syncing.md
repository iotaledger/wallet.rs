---
"nodejs-binding": patch
---

Send sync requests in chunks to prevent timeouts, make background sync not blocking the whole time.
Changed polling interval to wait after each sync operations, so it doesn't start immediately if the syncing takes longer than the polling interval.
