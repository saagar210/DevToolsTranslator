# Phase 7 UI Implementation

Date: 2026-02-22

## Scope Completed
- Full React desktop UI implemented in `apps/desktop-tauri/ui`.
- Global views implemented: Sessions, Live Capture, Findings, Exports, Settings, About/Diagnostics.
- Session subviews implemented: Overview, Timeline, Network, Console, Findings, Export.
- Evidence deep-link and highlight behavior implemented with exact-pointer and fallback handling.
- Tauri command adapter implemented with feature-gated Rust bindings (`desktop_shell`) and mock adapter for tests.
- Export UX implemented with explicit disabled actions until Phase 8 exporter wiring.

## Route Map
- `/sessions`
- `/live-capture`
- `/findings`
- `/exports`
- `/settings`
- `/diagnostics`
- `/sessions/:sessionId/overview`
- `/sessions/:sessionId/timeline`
- `/sessions/:sessionId/network`
- `/sessions/:sessionId/console`
- `/sessions/:sessionId/findings`
- `/sessions/:sessionId/export`

## Command Contracts (UI Adapter)
- Capture commands:
  - `ui_list_tabs`
  - `ui_start_capture`
  - `ui_stop_capture`
  - `ui_set_ui_capture`
- Query commands:
  - `ui_get_sessions`
  - `ui_get_session_overview`
  - `ui_get_timeline`
  - `ui_get_network`
  - `ui_get_console`
  - `ui_get_findings`
  - `ui_get_exports`
  - `ui_get_diagnostics`
- Evidence command:
  - `ui_resolve_evidence`

## Evidence Highlight Behavior
- Route query format:
  - `hl_kind`, `hl_id`, `hl_col`, `hl_ptr`, `hl_exact`, `hl_fallback`
- Resolution behavior:
  - If exact target exists, highlight exact row/cell container.
  - If exact target is unavailable, highlight nearest container and show fallback notice.
- Highlight timing:
  - Highlight class applied on navigation and removed after 4 seconds.
- Test coverage:
  - exact pointer highlight path
  - fallback message path

## Live Capture UX Notes
- Displays connection status, pairing port, pairing token, active session, tab list, and diagnostics stream.
- Start/Stop availability is derived from deterministic state via `buildLiveCaptureViewModel`.
- Explicit consent and UI-capture toggle are visible and actionable.

## Phase 7 Export-State Note
- At Phase 7 delivery time, Export views were implemented with policy-aware disabled actions pending Phase 8 backend wiring.
- Current live export behavior is tracked in `docs/PHASE8_READINESS.md`.

## Determinism and Privacy
- UI list ordering follows backend deterministic ordering constraints.
- Evidence routing/highlight uses stable route/query parameters.
- No privacy-mode escalation is introduced in Phase 7.
