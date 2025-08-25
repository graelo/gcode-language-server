# 0004 — Hover & Diagnostics (MVP)

Status: open

Goal
- Implement hover information and diagnostics for unknown commands and malformed parameters, flavor-aware.

Acceptance criteria
- Hover shows short or long descriptions based on the `--description` flag
- Diagnostics published for unknown commands with a clear message
- Hover and diagnostics use flavor metadata only (no network calls)

Tasks
- [ ] Implement hover resolver using document text and flavor command map
- [ ] Wire diagnostics engine to parser output
- [ ] Add unit tests for hover and diagnostics
