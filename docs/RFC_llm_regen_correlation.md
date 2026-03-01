# RFC: `llm_regen` Correlation Semantics v1

- Status: Proposed
- Date: 2026-02-22
- Owners: DevTools Translator Core
- Spec impact: No direct edits to `docs/SPEC_LOCK.md` in this RFC

## Context

`llm_regen` is implemented in correlation, but `docs/SPEC_LOCK.md` leaves parts of regeneration detection implicit (for example, tight timing boundaries and endpoint/provider matching details). This can lead to divergent implementations across crates and future phases.

## Problem

Without a concrete rule, two equivalent sessions may produce different interaction kinds (`llm_message` vs `llm_regen`) depending on implementation details, reducing determinism and fixture stability.

## Proposal

Adopt the following deterministic rule for v1 behavior:

1. Candidate pool is limited to interactions already classified as LLM-capable (`llm_message`) by host/path fingerprinting.
2. A candidate interaction is reclassified to `llm_regen` only when all conditions hold:
- same provider fingerprint and endpoint signature as the immediately prior LLM interaction in session order
- prior interaction is closed
- new interaction opens within `INTERACTION_CLOSE_IDLE_MS` of the prior close time
3. If any condition is missing/ambiguous, keep `llm_message`.
4. Tie-breaking and ordering remain deterministic using existing `(opened_at_ms, interaction_kind, interaction_id)` sort.

## Rationale

- Preserves deterministic behavior with existing constants.
- Avoids speculative regrouping when metadata is sparse.
- Keeps detector inputs stable for Phase 5+ snapshot workflows.

## Test Requirements

- Existing fixture test `fx_phase4_llm_regen` remains the canonical proof.
- Add/maintain assertions that shuffling raw input order does not alter regen classification.

## Rollout

- No migration required.
- No API schema changes required.
- If spec lock adopts this wording later, this RFC can be marked accepted and superseded by lock text.
