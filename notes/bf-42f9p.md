# bf-42f9p: Fix unused trait in embedding_service.rs

## Task
Fix unused trait warning in hoop-daemon/src/embedding_service.rs: EmbedderExt trait (line 458)

## Finding
The `EmbedderExt` trait was **already removed** in a previous commit:
- Commit `a452c70`: "fix(bf-2eulg): Remove unused EmbedderExt trait"
- This resolved bead `bf-2eulg`

## Verification
Ran `cargo clippy` on hoop-daemon:
- No unused trait warnings in `embedding_service.rs`
- No references to `EmbedderExt` trait in the codebase

## Outcome
This bead `bf-42f9p` is redundant - the issue was already fixed in commit `a452c70`.
The current codebase has no unused trait warnings in `embedding_service.rs`.
