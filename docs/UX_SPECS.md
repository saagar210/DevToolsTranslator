# UX Specifications v1.0

## Navigation Model
Global navigation tabs:
1. Sessions
2. Live Capture
3. Findings
4. Exports
5. Settings
6. About/Diagnostics

Session subviews:
1. Overview
2. Timeline
3. Network
4. Console
5. Findings
6. Export

## Shared State Model
Every screen must support:
- Loading
- Empty
- Success
- Error
- Disabled actions
- Focus-visible keyboard states

## Screen Specifications
### Sessions
- Purpose: list and filter captured sessions.
- Key elements: session table, status badge, privacy mode badge, duration, findings count.
- Empty state: explain capture prerequisites and quick action to Live Capture.
- Error state: storage/DB read failure with retry.

### Live Capture
- Purpose: pair extension, select tab, start/stop capture.
- Key elements: pairing token status, tab list, attach state, privacy mode selector.
- Loading: waiting for extension hello/tabs.
- Empty: no eligible tabs or extension unavailable.
- Error: websocket/pairing failure; actionable diagnostics link.

### Findings (global)
- Purpose: severity-ranked findings across sessions.
- Key elements: severity filters, detector pack toggles, confidence badges.
- Empty: no findings for selected filters.
- Error: analysis query failure and retry.

### Exports (global)
- Purpose: list generated bundles and integrity status.
- Key elements: export profile badge (share-safe/full), hash validation status, open-folder action.
- Empty: no exports yet.
- Error: filesystem or integrity read failure.

### Settings
- Purpose: defaults and guardrails.
- Key elements: default privacy mode, redaction profile, export defaults, diagnostics toggles.

### About/Diagnostics
- Purpose: health and troubleshooting.
- Key elements: app version, ws listener status, pairing state, buffer drops, recent errors.

## Session Subview Specifications
### Overview
- Session metadata, detector summary, top findings, quick export CTA.

### Timeline
- Event and interaction timeline with deterministic ordering.
- Click on interaction opens correlated network/console members.

### Network
- Request table with status, host/path, duration, size, and stream badges.
- Row detail pane with headers/timing/stream summary JSON.

### Console
- Console table with level, source, redacted message preview.
- Pattern tags for detector-linked entries.

### Findings (session)
- Finding cards with claim tree and evidence references.
- Severity and confidence visualization.
- Fix steps render from `fix_steps_json`.

### Export (session)
- Export mode selector defaulting to share-safe.
- Full export warning panel and gating reason if blocked.
- Progress and final integrity validation state.

## Evidence Deep-Linking And Highlight Behavior
1. User clicks evidence from claim.
2. Router deep-links to target view:
- `raw_event` -> timeline/event detail
- `net_row` -> network row and column/pointer detail
- `console` -> console row and optional pointer
- `derived_metric` -> finding detail metric panel
3. App attempts exact pointer highlight.
4. If exact pointer missing, app falls back to nearest container highlight and displays "Exact pointer unavailable" info notice.

### Highlight rules
- Scroll target into view.
- Apply temporary highlight for 4 seconds.
- Persist selected target in side panel until changed.

## Export UX And Warnings
### Default behavior
- Share-safe export preselected.
- Full export option disabled when `privacy_mode=metadata_only`.

### Warnings
- Full export warning includes explicit risk statement about sensitive payload inclusion.
- User must confirm acknowledgement before full export starts.

### Completion
- Show generated path, manifest summary, and integrity pass/fail.
- Provide quick action to open export folder.

## Accessibility And Input States
- Keyboard reachable controls across all views.
- Focus-visible style required for every interactive element.
- Disabled controls include explanation tooltip when blocked by privacy policy.
- Error banners are screen-reader announced.
