# Issue naming convention and workflow

This file defines how issues in `docs/issues/` are named, tracked, and referenced when collaborating with the assistant.

Filename format
- `NNNN-short-slug.md` where `NNNN` is a zero-padded numeric ID (e.g., `0001`) and `short-slug` is a brief dash-separated identifier.

Front-matter (required fields in the top of each issue file)
- Title: human-friendly title
- Status: `open` | `in-progress` | `done` | `blocked`
- Assignee: optional
- Priority: `P0`|`P1`|`P2`

Example filename: `docs/issues/0001-project-scaffold.md`

How to request work from the assistant
- Tell the assistant the issue ID (e.g., "Work on 0001"). The assistant will find the matching file and continue from the issue's TODOs.
- If the assistant modifies files to resolve the issue, it will update the issue file's `Status:` to `in-progress` and then to `done` when complete.

Issue lifecycle
- Created: `Status: open`
- In progress: assistant edits files and marks `Status: in-progress`
- Done: assistant updates `Status: done` and appends a short summary of changes and verification commands
- Archived: completed issues are moved to `docs/issues/done/` to keep the active issues directory clean

File organization
- Active issues: `docs/issues/NNNN-*.md` (status: `open`, `in-progress`, `blocked`)
- Completed issues: `docs/issues/done/NNNN-*.md` (status: `done`)
- Convention: `docs/issues/0000-issue-convention.md` (always stays in root)

Naming and referencing convention for quick chat commands
- "Work on 0001" — start or continue work on issue 0001
- "Status 0001" — ask the assistant to return the current status
- "Close 0001" — instruct assistant to finalize and set status to `done` (assistant will run tests/builds where applicable before closing)
- "Archive 0001" — move a completed issue to `docs/issues/done/` directory
