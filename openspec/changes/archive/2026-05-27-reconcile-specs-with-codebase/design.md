## Context

The specs in `openspec/specs/` were written incrementally during feature development. As the codebase matured, several patterns stabilized (Entity/EventEmitter, command channels, module structure) and new features were added (Configuration page, DevTest page, theme variants) without updating the corresponding specs. This design covers the approach for bringing all specs up to date.

## Goals / Non-Goals

**Goals:**
- Update every spec that diverges from the actual codebase behavior
- Preserve the existing spec format (Requirements + Scenarios with WHEN/THEN)
- Use delta specs (ADDED/MODIFIED/REMOVED) so changes are auditable
- Ensure specs accurately describe the architectural patterns in use

**Non-Goals:**
- No code changes — this is documentation only
- No new features or behavioral changes
- No restructuring of the openspec directory layout
- No changes to AGENTS.md (that happens post-archive)

## Decisions

### Use delta specs, not full rewrites

Each modified capability gets a delta spec file under `specs/<name>/spec.md` with `## MODIFIED Requirements` sections. This preserves the ability to audit what changed and why. At archive time, deltas are merged into the main specs.

### Only update specs with meaningful divergence

If a spec is functionally accurate but just uses slightly different wording, we leave it alone. We only update when the spec would mislead a developer about what the code does or how it's structured. For example:
- `pairing-status-indicator` — matches code, no delta needed
- `card-configuration-page` — matches code, no delta needed (CardRow entity pattern is accurately described)

### Capture architectural patterns in the relevant spec

Patterns like Entity/EventEmitter ownership, command channels, and the `status_strip` component are documented in the specs where they're used, not in a separate architecture spec. This keeps each spec self-contained.

## Risks / Trade-offs

**[Risk: Delta specs may be incomplete]** → Mitigation: After creating each delta, cross-reference against the actual source file to verify coverage.

**[Risk: Over-specifying implementation details]** → Specs should describe behavior and structure, not line-by-line code. We document patterns (e.g., "Entity with EventEmitter") not exact field names.

