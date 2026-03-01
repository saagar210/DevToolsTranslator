# RFC: Beginner Mode UX Contract

## Status
- Proposed

## Problem
- Current UI exposes too much implementation detail for first-time users.

## Decision
- Beginner mode is the default across desktop and extension surfaces.
- Technical details move behind explicit "Advanced details" disclosures.

## UX Contract
1. Use task-first language:
- Connect
- Choose a tab
- Start capture
- Review findings
- Export safely
2. Avoid protocol jargon in primary flows.
3. Always provide one primary action per state.
4. All changed surfaces must include:
- loading
- empty
- error
- success
- disabled
- focus-visible
5. Error copy must include:
- what failed
- what user can do next

## Accessibility Baseline
- WCAG 2.2 focus-visible for interactive controls.
- Color contrast that remains readable in default theme.
- Keyboard navigable controls on all new onboarding paths.
