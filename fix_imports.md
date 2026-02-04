# File Share Module Import Fixes

The file_share module has systematic import path issues. All files within `src/file_share/` are using `crate::` when they should use `crate::file_share::` for cross-module imports.

## Files Needing Import Fixes

### 1. Trust Module
- `src/file_share/trust/approval.rs`: Change `use crate::Result` → `use crate::file_share::Result`
- `src/file_share/trust/storage.rs`: Change `use crate::{FileShareError, Result}` → `use crate::file_share::{FileShareError, Result}`
- `src/file_share/trust/mod.rs`: Change `use crate::Result` → `use crate::file_share::Result`

### 2. Transfer Module
- `src/file_share/transfer/sender.rs`: 
  - Change `use crate::protocol::` → `use crate::file_share::protocol::`
  - Add `use tokio::io::AsyncSeekExt;`

### 3. Connection Module
- `src/file_share/connection/listener.rs`: Change `crate::protocol::` → `crate::file_share::protocol::`
- `src/file_share/connection/manager.rs`: Change `crate::protocol::` → `crate::file_share::protocol::`

### 4. Transfer Orchestrator
- `src/file_share/transfer/orchestrator.rs`: 
  - Change all `crate::protocol::` → `crate::file_share::protocol::`
  - Change all `crate::api::` → `crate::file_share::api::`

### 5. API Commands
- `src/file_share/api/commands.rs`:
  - Change `crate::trust::TrustedDevice` → `crate::file_share::trust::TrustedDevice`
  - Change `crate::connection::ConnectionInfo` → `crate::file_share::connection::ConnectionInfo`

### 6. Discovery
- `src/file_share/discovery/mdns.rs`: Fix ed25519_dalek PublicKey usage (use VerifyingKey instead)

## Quick Fix Pattern

Replace all occurrences in file_share module:
- `use crate::Result` → `use crate::file_share::Result`
- `use crate::FileShareError` → `use crate::file_share::FileShareError`
- `use crate::protocol` → `use crate::file_share::protocol`
- `use crate::api` → `use crate::file_share::api`
- `use crate::trust` → `use crate::file_share::trust`
- `use crate::connection` → `use crate::file_share::connection`
- `crate::protocol::` → `crate::file_share::protocol::`
- `crate::api::` → `crate::file_share::api::`

## Note

These are all backend file_share module issues. The UI (Phase 3) is complete and has no errors. These import fixes are needed to make the backend compile, but don't affect the UI integration work.
