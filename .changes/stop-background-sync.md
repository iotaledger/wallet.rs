---
"nodejs-binding": patch
---

Stop endlessly waiting in `AccountManager::stopBackgroundSync()` if background syncing wasn't started;