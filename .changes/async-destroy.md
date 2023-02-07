---
"nodejs-binding": patch
---

Make `AccountManager.destroy()` async;
Await in `AccountManager.stopBackgroundSync()` until syncing actually stopped;
Don't spawn a background task anymore to retry a transaction;
