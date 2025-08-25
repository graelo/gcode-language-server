# 0006 — Incremental parsing & performance for large files

Status: open

Goal
- Ensure the server can handle large files (target ~20 MB) with responsive edits using incremental parsing strategies.

Acceptance criteria
- Edits result in near-instant diagnostics/response on typical machines
- Memory usage remains bounded and reasonable

Tasks
- [ ] Implement incremental parse of changed ranges
- [ ] Add benchmarks and CI checks for performance
- [ ] Optimize hotspot allocations
