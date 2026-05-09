//! Multi-operator concurrency support (§19)
//!
//! HOOP supports multiple simultaneous operators through functionality
//! distributed across several modules, not centralized in one place.
//!
//! ## Architecture
//!
//! ### Role-based access control (auth.rs)
//! - Two-role model: viewer (read-only) and drafter (read + create beads)
//! - Tailscale identity-based role assignment via `tailscale whois`
//! - Route-level middleware enforces permissions at schema boundary
//!
//! ### Stitch draft concurrency (api_draft_queue.rs)
//! - Drafts are server-persisted from the moment the operator opens a draft form
//! - Two operators drafting in the same project: both accepted, no conflicts
//! - Presence indicators show "operator X is drafting in project Y"
//!
//! ### Reflection Ledger concurrency (reflection_detector.rs)
//! - Proposals deduplicated on create (content hash)
//! - Approvals are single-operator actions with approver tracking
//! - Rejection tracking prevents immediate re-proposal
//!
//! ### Agent session ownership (agent_session.rs)
//! - Each operator has their own agent session
//! - View-only operators can read others' transcripts but not inject
//! - Audit log attributes actions to the operator whose agent drafted the Stitch
//!
//! ### Presence tracking (api_presence.rs)
//! - Optional presence indicators per project and per Stitch
//! - Operator-toggleable privacy setting (show me / hide me)
//! - Does not block writes; multiple operators can view the same Stitch
//!
//! ### Conflict resolution (collision_detector.rs)
//! - Two operators drafting Stitches targeting the same workspace: both submit
//! - Already-Started Detection alerts before submission
//! - Offers to combine drafts if both are still pending
//!
//! ## Per-operator UI state
//!
//! UI state is client-side and scoped per operator via browser localStorage
//! and session cookies. The server does not manage per-operator UI state.
//!
//! ## Success criteria (from plan.md §7)
//!
//! - ✅ Two operators see consistent state (shared fleet.db, event streams)
//! - ✅ Viewer role cannot access bead-creation endpoint (auth.rs middleware)
//! - ✅ README enables stranger to run HOOP in <30 min (README.md quickstart)
