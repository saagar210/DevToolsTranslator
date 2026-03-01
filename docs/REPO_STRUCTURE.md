# Repository Structure v1.0

## Proposed Tree
```text
/Users/d/Projects/DevToolsTranslator
├── docs/
│   ├── PRODUCT.md
│   ├── ARCHITECTURE.md
│   ├── SPEC_LOCK.md
│   ├── DATA_MODEL.md
│   ├── DETECTORS.md
│   ├── UX_SPECS.md
│   ├── EXPORTS.md
│   ├── TEST_PLAN.md
│   ├── IMPLEMENTATION_PLAN.md
│   └── REPO_STRUCTURE.md
├── apps/
│   ├── desktop-tauri/
│   │   ├── src-tauri/
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   └── ui/
│   └── extension-mv3/
│       ├── src/
│       └── manifest.json
├── crates/
│   ├── dtt-core/
│   ├── dtt-storage/
│   ├── dtt-correlation/
│   ├── dtt-detectors/
│   ├── dtt-export/
│   └── dtt-integrity/
├── packages/
│   ├── schemas/
│   ├── shared-types/
│   └── fixture-tools/
├── fixtures/
│   ├── raw/
│   ├── expected/
│   └── exports/
├── tests/
│   ├── e2e/
│   ├── integration/
│   └── snapshot/
├── config/
│   ├── detectors.v1.json
│   ├── patterns.console.v1.json
│   ├── telemetry.filters.v1.json
│   └── llm.fingerprints.v1.json
├── registry.v1.json
├── Cargo.toml
├── package.json
├── pnpm-workspace.yaml
└── .codex/
    └── verify.commands
```

## Rationale
### apps/
Contains product runtime entry points:
- `desktop-tauri` for local desktop orchestration and UI shell.
- `extension-mv3` for Chrome capture integration.

### crates/
Isolates Rust backend domains into testable modules:
- core contracts
- storage and migrations
- interaction correlation
- detector execution
- export generation
- integrity hashing

### packages/
Shared TS contracts/tooling for schema parity and fixture generation utilities.

### fixtures/
Fixture-first quality model:
- `raw`: captured session fixtures.
- `expected`: canonical detector and evidence outputs.
- `exports`: expected bundle snapshots and integrity files.

### tests/
Cross-layer validation:
- `e2e`: full capture-to-export scenarios.
- `integration`: module interoperability checks.
- `snapshot`: deterministic output assertions.

### config/ and registry
- Detector/runtime configuration is externalized and versioned.
- Supports deterministic configuration and easier future extension without code churn.

## Boundary Rules
- No implementation code in docs/config-only phases.
- All public contracts must originate from shared schema definitions.
- Exports and detector outputs must be reproducible from fixtures.

## Evolution Policy
- Additive structure changes allowed in v1.x.
- Breaking path/layout changes require major spec revision and migration notes.
