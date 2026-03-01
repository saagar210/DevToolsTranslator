# RFC: Pairing v2 User-Friendly Flow

## Status
- Proposed

## Problem
- The current localhost pairing flow is technically correct but too manual for beginner users.
- Users should not need to repeatedly enter a pairing port/token after first trust.

## Goals
- Preserve existing websocket transport and envelope compatibility.
- Enable one-click "Find Desktop App" and trusted auto-reconnect.
- Keep explicit capture consent as a hard gate.

## Non-Goals
- Replacing websocket transport with native messaging in this phase.
- Relaxing privacy mode and consent constraints.

## Decision
- Keep websocket transport on localhost.
- Introduce pairing UX state and trusted device memory.
- Keep manual token/port entry in an advanced fallback panel.

## Proposed Flow
1. User opens popup and clicks **Find Desktop App**.
2. Extension scans allowed pairing ports and attempts authenticated websocket handshake.
3. On success, extension marks device trusted and saves pairing details.
4. Extension auto-reconnects using trusted details on startup/restart.
5. If reconnect fails, popup shows plain-language recovery actions and advanced fallback.

## Backward Compatibility
- Existing control envelope shape remains valid.
- Existing `cmd.*` and `evt.*` types continue to work.
- Existing manual pairing still works as fallback.

## Security Notes
- Token remains required for websocket command channel.
- No change to explicit capture consent requirement.
- Sensitive header/body redaction behavior remains unchanged.
