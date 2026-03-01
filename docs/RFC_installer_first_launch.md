# RFC: Installer-First Launch and Handoff

## Status
- Proposed

## Problem
- Beginner users should not depend on terminal startup to use the app.

## Decision
- Installer/app-bundle launch becomes the primary path.
- Terminal startup remains an advanced developer path only.
- Extension adds an "Open Desktop App" helper action with fallback guidance.

## Behavior
1. Desktop docs lead with packaged app launch.
2. Extension attempts desktop handoff via deep link first.
3. If deep link/open fails, extension shows install/open instructions.
4. Pairing token/port remain visible only in advanced diagnostics.

## Constraints
- Existing desktop core behavior and spec lock stay backward compatible.
- No changes to capture consent rules.
