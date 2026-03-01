# DevTools Translator

Local-first tooling for capturing Chrome DevTools CDP events, normalizing/correlating/analyzing sessions, and reviewing findings in a desktop UI.

## Beginner Quick Start (No Terminal Required)
1. Open the Desktop App from your Applications/Start Menu.
2. In Chrome, open the DevTools Translator extension popup.
3. Click `Find Desktop App`.
4. If prompted, click `Connect`.
5. In the extension popup, enable `I allow capture for this browser`.
6. In Desktop App, open `Live Capture`, choose a tab, then click `Start`.
7. When done, click `Stop Capture` and review `Findings`.

## Repo Layout
- `apps/extension-mv3`: Chrome MV3 capture extension.
- `apps/desktop-tauri/src-tauri`: Rust desktop backend and WS bridge.
- `apps/desktop-tauri/ui`: React desktop UI.
- `crates/dtt-core`: shared Rust contracts.
- `crates/dtt-storage`: SQLite schema, ingest, normalization, correlation orchestration, UI queries.
- `crates/dtt-correlation`: deterministic interaction correlation engine.
- `crates/dtt-detectors`: detector engine and built-in detectors.
- `fixtures`: fixture streams and expected outputs.
- `docs/SPEC_LOCK.md`: authoritative specification.

## Setup (Advanced / Developer)
1. Install dependencies:
```bash
pnpm install
```
2. Ensure Rust toolchain is available:
```bash
cargo --version
```

## Run (Advanced / Developer)
### Desktop UI (web/dev shell)
```bash
pnpm --filter @dtt/desktop-ui dev
```

### Desktop UI test/build
```bash
pnpm --filter @dtt/desktop-ui test
pnpm --filter @dtt/desktop-ui build
```

### Extension build
```bash
pnpm --filter @dtt/extension build
```
Load extension from:
- `apps/extension-mv3/dist`

### Rust backend tests/build
```bash
cargo test -p dtt-desktop-core
cargo test -p dtt-storage
cargo build -p dtt-desktop-core
```

### Export workflow (Phase 8)
1. Capture and complete a session (Live Capture).
2. Open `/exports` or `/sessions/:sessionId/export` in the desktop UI.
3. Run `Generate Share-Safe Export` (default profile).
4. Run `Generate Full Export` only when session privacy mode is not `metadata_only`.
5. Use export list actions:
- `Validate` checks manifest/index/integrity chain.
- `Open Export Folder` opens when shell support is available, otherwise shows fallback path text.

### Release and Bundle Inspect (Phase 10)
1. Desktop release dry-run:
```bash
pnpm run release:desktop:mac:dry-run -- --version 0.1.0-beta.1
```
2. Extension beta dry-run packaging:
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/package_extension_beta.mjs --version 0.1.0-beta.1 --dry-run
```
3. CI internal beta workflow:
- `/Users/d/Projects/DevToolsTranslator/.github/workflows/release-internal-beta.yml` (`workflow_dispatch`)
4. Offline inspect in UI:
- open `/exports`
- use `Open Bundle Inspect` with a bundle zip path
- review overview/findings and resolve evidence refs in read-only mode.

### Multi-Platform Internal Beta + Perf (Phase 11)
1. Desktop release matrix dry-run:
```bash
pnpm run release:desktop:mac:dry-run -- --version 0.1.0-beta.1
pnpm run release:desktop:windows:dry-run -- --version 0.1.0-beta.1
pnpm run release:desktop:linux:dry-run -- --version 0.1.0-beta.1
```
2. Extension beta packaging dry-run:
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/package_extension_beta.mjs --version 0.1.0-beta.1 --dry-run
```
3. Perf/reliability gate:
```bash
pnpm run perf:phase11:gate
```
4. Diagnostics UI (Phase 11 additions):
- `/diagnostics` now shows 24h reliability KPI totals and trend buckets.
- `/diagnostics` supports starting/listing local perf runs.

### Staged Public Prerelease + OTLP-Optional Telemetry (Phase 12)
1. Staged public promotion dry-run workflow:
- `/Users/d/Projects/DevToolsTranslator/.github/workflows/release-staged-public-prerelease.yml`
- required inputs: `version`, `promote_from_internal_run_id`, `notes`, `dry_run`
 - local equivalent dry-run command:
```bash
pnpm run release:staged-public:dry-run -- --version 0.1.0-beta.12 --promote-from-internal-run-id internal_0.1.0-beta.12
```
2. Manual smoke gate:
- non-dry-run promotion is blocked unless `/Users/d/Projects/DevToolsTranslator/docs/PHASE6_SMOKE_EVIDENCE.md` contains dated pass evidence.
- required marker format:
`interactive_chrome_manual: pass|date=YYYY-MM-DD|observer=<name>`
3. Telemetry mode:
- default is `local_only`
- optional OTLP is configured in Diagnostics (`ui_set_telemetry_settings`) and can be exercised with `Run Telemetry Export`.
4. Endurance/perf lane:
```bash
pnpm run perf:phase12:endurance:ci
```
5. Phase 12 completion evidence:
- `/Users/d/Projects/DevToolsTranslator/docs/PHASE12_RELEASE_PROMOTION_REPORT.md`

### Public Extension Rollout + Signed Updates + Audits (Phase 13)
1. Package extension for Chrome Web Store public rollout (dry-run):
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/package_extension_public.mjs --dry-run --version 0.1.0-beta.13
```
2. Publish extension rollout stage (dry-run):
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/publish_extension_cws.mjs --dry-run --version 0.1.0-beta.13 --stage pct_5
```
3. Generate signed updater feed (dry-run):
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/generate_updater_feed.mjs --dry-run --version 0.1.0-beta.13 --channel staged_public_prerelease
```
4. Diagnostics/UI additions:
- `/exports` includes extension public rollout controls and desktop update eligibility/apply controls.
- `/diagnostics` includes telemetry audit history and perf anomaly tables.
5. Endurance lanes:
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/perf/run_endurance_suite.mjs --mode ci
node /Users/d/Projects/DevToolsTranslator/scripts/perf/run_endurance_suite.mjs --mode nightly
node /Users/d/Projects/DevToolsTranslator/scripts/perf/run_endurance_suite.mjs --mode weekly
```
6. Phase 13 completion evidence:
- `/Users/d/Projects/DevToolsTranslator/docs/PHASE13_PUBLIC_CHANNEL_REPORT.md`

### Rollout Ops Automation + Health Scorecards (Phase 14)
0. Non-dry-run promotion readiness precheck:
```bash
pnpm run release:promotion:readiness -- --version 0.1.0-beta.14 --channel staged_public_prerelease
```
1. Evaluate extension rollout stage (dry-run):
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/evaluate_extension_stage.mjs --dry-run --version 0.1.0-beta.14 --stage pct_5
```
2. Build extension stage-advance approval artifact:
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/advance_extension_stage.mjs --dry-run --version 0.1.0-beta.14 --from-stage pct_5 --to-stage pct_25
```
3. Evaluate updater rollout stage (dry-run):
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/evaluate_updater_rollout.mjs --dry-run --version 0.1.0-beta.14 --channel public_stable --stage pct_5
```
4. Build updater stage-advance approval artifact:
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/advance_updater_rollout.mjs --dry-run --version 0.1.0-beta.14 --channel public_stable --from-stage pct_5 --to-stage pct_25
```
5. Generate updater feed with rollout/evidence metadata:
```bash
node /Users/d/Projects/DevToolsTranslator/scripts/release/generate_updater_feed.mjs --dry-run --version 0.1.0-beta.14 --channel staged_public_prerelease --stage pct_25 --evidence-pack-path dist/releases/evidence/updater/0.1.0-beta.14/pct_25
```
6. Workflow automation:
- Extension controller workflow: `/Users/d/Projects/DevToolsTranslator/.github/workflows/release-extension-stage-controller.yml`
- Extension public workflow now consumes controller approval artifacts.
- Staged public prerelease workflow now enforces release health scorecard `pass` for non-dry-run promotion.
7. UI additions:
- `/exports` includes Rollout Ops controls (evaluate/advance/tick), global scorecard panel, and compliance evidence pack table.
- Evidence packs are listed and queryable from backend commands.

### Retention and deletion (Phase 9)
1. Open `/settings` in the desktop UI.
2. Configure retention policy:
- `enabled`
- `retain_days`
- `max_sessions`
- `delete_exports`
- `delete_blobs`
3. Use `Run Retention (Dry Run)` to preview candidate/deletion counts.
4. Use `Run Retention (Apply)` to execute deletion cascade for eligible ended sessions.
5. Sessions can be deleted directly from `/sessions` with confirmation; running sessions are blocked by policy.

## Canonical Verification
Canonical commands are defined in `/.codex/verify.commands`.
Run all gates:
```bash
pnpm -r lint
pnpm -r typecheck
pnpm -r test
pnpm -r build
pnpm --filter @dtt/extension lint
pnpm --filter @dtt/extension test
pnpm --filter @dtt/extension build
pnpm --filter @dtt/desktop-ui test
pnpm --filter @dtt/desktop-ui build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo test -p dtt-desktop-core
cargo test -p dtt-storage
cargo build -p dtt-desktop-core
cargo test -p dtt-detectors
cargo test -p dtt-correlation
cargo test -p dtt-export
cargo test -p dtt-integrity
cargo check -p dtt-desktop-core --features desktop_shell
cargo test -p dtt-desktop-core --features desktop_shell
cargo test -p dtt-storage reliability -- --nocapture
cargo test -p dtt-storage perf -- --nocapture
node scripts/release/release_desktop_mac.mjs --dry-run --version 0.1.0-beta.1
node scripts/release/release_desktop_windows.mjs --dry-run --version 0.1.0-beta.1
node scripts/release/release_desktop_linux.mjs --dry-run --version 0.1.0-beta.1
node scripts/release/package_extension_beta.mjs --dry-run --version 0.1.0-beta.1
node scripts/release/package_extension_public.mjs --dry-run --version 0.1.0-beta.13
node scripts/release/publish_extension_cws.mjs --dry-run --version 0.1.0-beta.13 --stage pct_5
node scripts/release/generate_updater_feed.mjs --dry-run --version 0.1.0-beta.13 --channel staged_public_prerelease
node scripts/perf/run_perf_gate.mjs
node scripts/perf/run_endurance_suite.mjs --mode ci
```

## Troubleshooting
- `Cannot find module '@dtt/shared-types'` in desktop UI:
  - Run `pnpm install` at repo root.
- `vite/client` or `vitest/globals` TS errors:
  - Ensure desktop UI dev dependencies are installed and rerun `pnpm --filter @dtt/desktop-ui typecheck`.
- UI evidence highlight test failures in jsdom:
  - The code guards `scrollIntoView` for test/runtime compatibility; rerun `pnpm --filter @dtt/desktop-ui test`.
- WS bridge command errors (`bridge_unavailable`):
  - Start/attach the desktop bridge before capture commands.

## Fixture Workflow
1. Ingest fixture raw events in storage tests.
2. Run normalization, correlation, and detector analysis via package tests.
3. Compare outputs against `fixtures/expected` snapshot files.
4. For new fixtures:
- Add raw stream under `fixtures/raw`.
- Add expected outputs under `fixtures/expected`.
- Add deterministic replay assertions in `crates/dtt-storage` tests.

## Current Phase Status
- Phase 6 transport/capture implemented with automated smoke coverage; interactive browser smoke checklist tracked in `docs/PHASE6_SMOKE_EVIDENCE.md`.
- Phase 7 desktop UI implemented (global + session views, evidence routing/highlight, feature-gated Tauri command integration).
- Phase 8 export engine implemented (share-safe/full gating, manifest/index/report/integrity bundle, offline evidence resolution, UI export actions).
- Phase 9 hardening implemented (retention/deletion cascades, diagnostics persistence, reconnect hardening, expanded regression/fixture gates); report in `docs/PHASE9_HARDENING_REPORT.md`.
- Phase 10 operationalization implemented (release runs + bundle inspections persistence, release scripts/workflow, offline bundle inspect UI); report in `docs/PHASE10_RELEASE_REPORT.md`.
- Phase 11 implemented (multi-platform internal-beta release matrix, persisted reliability telemetry, perf-run storage/query/UI, perf regression scripts/workflows, high-volume fixtures); report in `docs/PHASE11_IMPLEMENTATION_REPORT.md`.
- Phase 12 implemented (staged public prerelease promotion workflow, signing/provenance gating, OTLP-optional telemetry exports, endurance suites and trend surfaces); report in `docs/PHASE12_RELEASE_PROMOTION_REPORT.md`.
- Phase 13 implemented (extension public staged rollout workflows, updater eligibility/apply APIs, telemetry privacy audit persistence, multi-day anomaly surfaces); report in `docs/PHASE13_PUBLIC_CHANNEL_REPORT.md`.
- Phase 14 implemented (controller-driven extension/updater stage decisions, compliance evidence packs, release health scorecards, rollout ops workflows/UI); report in `docs/PHASE14_ROLLOUT_AUTOMATION_REPORT.md`.
- Final release closeout still requires human-run interactive Chrome manual smoke checklist in `docs/PHASE6_SMOKE_EVIDENCE.md`.
