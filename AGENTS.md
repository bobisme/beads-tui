# beads-tui

Project type: tui
Tools: `beads`, `maw`, `crit`, `botbus`, `botty`
Reviewer roles: security

<!-- Add project-specific context below: architecture, conventions, key files, etc. -->


This document provides guidance for AI coding agents working on `beads-tui` (`bu`).

## Project Overview

`bu` is a Rust/Ratatui TUI for viewing and managing beads (issues). It reads from SQLite (`.beads/beads.db`) and shells out to `br` (beads_rust) for mutations.

## Quick Reference

```bash
# Build and run
just build
cargo run

# Run tests
just test

# Check formatting and lints
cargo fmt --check
cargo clippy
```

---

## Agent Communication

This project uses BotBus for agent coordination. BotBus uses global storage (~/.local/share/botbus/) shared across all projects.

### Quick Start

```bash
# Set your identity (once per session)
export BOTBUS_AGENT=$(botbus generate-name)  # e.g., "swift-falcon"
# Or choose your own: export BOTBUS_AGENT=my-agent-name

# Check what's happening
botbus status              # Overview: agents, channels, claims
botbus history             # Recent messages in #general
botbus agents              # Who's been active

# Communicate
botbus send general "Starting work on X"
botbus send general "Done with X, ready for review"
botbus send @other-agent "Question about Y"

# Coordinate file access (claims use absolute paths internally)
botbus claim "src/ui/**" -m "Working on UI components"
botbus check-claim src/ui/list.rs   # Check before editing
botbus release --all                 # When done
```

### Best Practices

1. **Set BOTBUS_AGENT** at session start - identity is stateless
2. **Run `botbus status`** to see current state before starting work
3. **Claim files** you plan to edit - overlapping claims are denied
4. **Check claims** before editing files outside your claimed area
5. **Send updates** on blockers, questions, or completed work
6. **Release claims** when done - don't hoard files

### Channel Conventions

- `#general` - Default channel for cross-project coordination
- `#beads-tui` - Project-specific updates
- `@agent-name` - Direct messages for specific coordination

Channel names: lowercase alphanumeric with hyphens (e.g., `my-channel`)

### Message Conventions

Keep messages concise and actionable:
- "Starting work on bd-xyz: Add foo feature"
- "Blocked: need clarification on UI layout"
- "Question: should status popup use a modal or inline?"
- "Done: implemented bar, tests passing"

### Waiting for Replies

```bash
# After sending a DM, wait for reply
botbus send @other-agent "Can you review this?"
botbus wait -c @other-agent -t 60  # Wait up to 60s for reply

# Wait for any @mention of you
botbus wait --mention -t 120
```

---

## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) for issue tracking. Issues are stored in `.beads/` and tracked in version control.

**Note:** `br` (beads_rust) is non-invasive and never executes git/jj commands directly. After running `br sync --flush-only`, you must manually commit changes.

### Essential Commands

```bash
# View issues
br list --status=open     # All open issues
br ready                  # Issues ready to work (no blockers)
br show <id>              # Full issue details with dependencies

# Create and update
br create --title="..." --type=task --priority=2
br update <id> --status=in_progress
br close <id> --reason="Completed"
br close <id1> <id2>      # Close multiple issues at once

# Sync to version control
br sync --flush-only      # Export to JSONL (does NOT run git/jj commands)
jj                        # Then commit with jj
```

### Workflow Pattern

1. **Start**: Run `br ready` to find actionable work
2. **Claim**: Use `br update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id> --reason="..."`
5. **Sync**: Run `br sync --flush-only`, then commit with jj

### Issue Quality

When creating or updating issues, always include:
- **Description**: What the issue is about, context, and acceptance criteria
- **Labels**: Use `--add-label` to categorize (e.g., `ui`, `data`, `bug`, `enhancement`)

```bash
br create --title="Add foo feature" --type=task --priority=2
br update <id> --description="Detailed description here" --add-label=ui --add-label=enhancement
```

---

## Version Control: jj (Jujutsu)

This project uses `jj` instead of `git`. Key differences:

- Working copy is always a commit (the `@` commit)
- No staging area - all changes are part of `@`
- Use `jj describe` to set commit message, `jj new` to create next change
- Bookmarks instead of branches

### Common Commands

```bash
jj status                 # See current changes (like git status)
jj diff                   # See what changed
jj log                    # View history
jj describe -m "message"  # Set commit message for working copy
jj new                    # Create new change on top of current
jj bookmark create name   # Create a bookmark (like a branch)
jj git push               # Push to remote
```

### Typical Workflow

```bash
# Make changes to files...
jj status                 # Review changes
jj describe -m "feat(ui): add status indicators"
jj new                    # Start next change
```

---

## Commit Conventions

Use [semantic commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Scopes**: `ui`, `data`, `app`, `cli`, etc.

**Always include** the `Co-Authored-By` trailer when AI assists with commits.

Examples:
- `feat(ui): add two-pane layout with resizable split`
- `fix(data): handle null description in bead parsing`
- `docs: update README with installation instructions`
- `refactor(ui): extract theme colors to separate module`

### Semantic Versioning

**IMPORTANT**: Update the version in `Cargo.toml` according to [semver](https://semver.org/) for every commit to main:

- **MAJOR** (x.0.0): Breaking changes (incompatible API changes, major UI overhauls)
- **MINOR** (0.x.0): New features (backward-compatible functionality additions)
  - Examples: `feat(ui):`, `feat(data):`, new commands, new UI panels
- **PATCH** (0.0.x): Bug fixes and minor improvements (backward-compatible bug fixes)
  - Examples: `fix():`, `refactor():`, `docs():`, `style():`

When in doubt:
- Adding features → bump MINOR
- Fixing bugs/refactoring → bump PATCH
- Breaking existing behavior → bump MAJOR (use sparingly)

**Update `Cargo.toml` version in the same commit** where you make the changes.

### Merge and Release

After changes are ready (tests pass, clippy clean, formatted):

```bash
# Pre-flight checks
cargo fmt
cargo clippy -- -D warnings
just test

# Bump version in Cargo.toml
# e.g., 0.3.1 to 0.4.0

# Commit with jj
jj describe -m "feat(scope): description

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# Move main bookmark to current commit
jj bookmark set main -r @

# IMPORTANT: Tag with explicit commit hash, not git HEAD
# Get the commit hash from jj log
COMMIT_HASH=$(git log -1 --format=%h)
git tag vX.Y.Z $COMMIT_HASH -m "feat(scope): description"

# Push bookmark and tags
jj git push && git push --tags

# Install locally
cargo install --path .

# Verify
bu --version

# Announce on botbus
botbus --agent <your-agent> send beads-tui "Released vX.Y.Z - [summary of changes]"
```

**Common Mistake to Avoid:**

❌ **WRONG**: `git tag vX.Y.Z` (tags whatever git HEAD points to, which may be stale)

✅ **CORRECT**: `git tag vX.Y.Z $COMMIT_HASH` (tags the specific commit you just created)

When using jj, `jj bookmark set main` updates jj's internal state but doesn't immediately move git's HEAD. Always use the explicit commit hash when tagging to ensure you tag the correct commit.

---

## Code Style

### Rust Conventions

- Use `rustfmt` defaults (run `cargo fmt`)
- Follow Clippy suggestions (run `cargo clippy`)
- Prefer `anyhow::Result` for error handling in application code
- Use `thiserror` for library-style error types if needed
- Document public APIs with `///` doc comments

### Project Structure

```
src/
├── main.rs          # Entry point, CLI parsing
├── app.rs           # App state machine and event loop
├── event.rs         # Input event handling
├── ui/
│   ├── mod.rs       # UI module exports
│   ├── layout.rs    # Main layout (two-pane)
│   ├── list.rs      # Bead list widget
│   ├── detail.rs    # Detail panel widget
│   ├── modal.rs     # Modal dialogs (create, help)
│   └── theme.rs     # Color themes
└── data/
    ├── mod.rs       # Data module exports
    ├── bead.rs      # Bead struct and types
    ├── sqlite.rs    # SQLite reader
    └── br.rs        # br CLI wrapper
```

### Testing

- Write unit tests in the same file using `#[cfg(test)]` module
- Integration tests go in `tests/` directory
- Run `just test` before committing

---

## Tools

### Recommended Development Setup

```bash
# Watch for changes and rebuild
cargo watch -x check -x test -x run

# Or just check on save
cargo watch -x check
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run specific test with output
cargo test test_name -- --nocapture
```


<!-- botbox:managed-start -->
## Botbox Workflow

This project uses the botbox multi-agent workflow.

### Identity

Every command that touches bus or crit requires `--agent <name>`.
Use `<project>-dev` as your name (e.g., `terseid-dev`). Agents spawned by `agent-loop.sh` receive a random name automatically.
Run `bus whoami --agent $AGENT` to confirm your identity.

### Lifecycle

**New to the workflow?** Start with [worker-loop.md](.agents/botbox/worker-loop.md) — it covers the complete triage → start → work → finish cycle.

Individual workflow docs:

- [Close bead, merge workspace, release claims, sync](.agents/botbox/finish.md)
- [groom](.agents/botbox/groom.md)
- [Verify approval before merge](.agents/botbox/merge-check.md)
- [Validate toolchain health](.agents/botbox/preflight.md)
- [Report bugs/features to other projects](.agents/botbox/report-issue.md)
- [Reviewer agent loop](.agents/botbox/review-loop.md)
- [Request a review](.agents/botbox/review-request.md)
- [Handle reviewer feedback (fix/address/defer)](.agents/botbox/review-response.md)
- [Claim bead, create workspace, announce](.agents/botbox/start.md)
- [Find work from inbox and beads](.agents/botbox/triage.md)
- [Change bead status (open/in_progress/blocked/done)](.agents/botbox/update.md)
- [Full triage-work-finish lifecycle](.agents/botbox/worker-loop.md)

### Quick Start

```bash
AGENT=<project>-dev   # or: AGENT=$(bus generate-name)
bus whoami --agent $AGENT
br ready
```

### Beads Conventions

- Create a bead for each unit of work before starting.
- Update status as you progress: `open` → `in_progress` → `closed`.
- Reference bead IDs in all bus messages.
- Sync on session end: `br sync --flush-only`.
- **Always push to main** after completing beads (see [finish.md](.agents/botbox/finish.md)).
- **Release after features/fixes**: If the batch includes user-visible changes (not just chores), follow the project's release process (version bump → tag → announce).

### Beads Quick Reference

Beads are **project-local** — always `cd` to the project directory first.

| Operation | Command |
|-----------|---------|
| View ready work | `br ready` |
| Show bead | `br show <id>` |
| Create | `br create --actor $AGENT --owner $AGENT --title="..." --type=task --priority=2` |
| Start work | `br update --actor $AGENT <id> --status=in_progress` |
| Add comment | `br comments add --actor $AGENT --author $AGENT <id> "message"` |
| Close | `br close --actor $AGENT <id>` |
| Add labels | `br update --actor $AGENT <id> --labels=foo,bar` |
| Add dependency | `br dep add --actor $AGENT <blocked> <blocker>` |
| Block | `br update --actor $AGENT <id> --status=blocked` |
| Sync | `br sync --flush-only` |

**Required flags**: `--actor $AGENT` on all mutations, `--author $AGENT` on comments.

### Mesh Protocol

- Include `-L mesh` on bus messages.
- Claim bead: `bus claims stake --agent $AGENT "bead://$BOTBOX_PROJECT/<bead-id>" -m "<bead-id>"`.
- Claim workspace: `bus claims stake --agent $AGENT "workspace://$BOTBOX_PROJECT/$WS" -m "<bead-id>"`.
- Claim agents before spawning: `bus claims stake --agent $AGENT "agent://role" -m "<bead-id>"`.
- Release claims when done: `bus claims release --agent $AGENT --all`.

### Spawning Agents

1. Check if the role is online: `bus agents`.
2. Claim the agent lease: `bus claims stake --agent $AGENT "agent://role"`.
3. Spawn with an explicit identity (e.g., via botty or agent-loop.sh).
4. Announce with `-L spawn-ack`.

### Reviews

- Use `crit` to create reviews and `@<project>-<role>` mentions to spawn reviewers.
- To request a security review:
  1. `crit reviews request <review-id> --reviewers $PROJECT-security --agent $AGENT`
  2. `bus send --agent $AGENT $PROJECT "Review requested: <review-id> @$PROJECT-security" -L review-request`
  (The @mention in the bus message triggers the auto-spawn hook)
- Reviewer agents loop until no pending reviews remain (see review-loop doc).

### Cross-Project Feedback

When you encounter issues with tools from other projects:

1. Query the `#projects` registry: `bus inbox --agent $AGENT --channels projects --all`
2. Find the project entry (format: `project:<name> repo:<path> lead:<agent> tools:<tool1>,<tool2>`)
3. Navigate to the repo, create beads with `br create`
4. Post to the project channel: `bus send <project> "Filed beads: <ids>. <summary> @<lead>" -L feedback`

See [report-issue.md](.agents/botbox/report-issue.md) for details.

### Stack Reference

| Tool | Purpose | Key commands |
|------|---------|-------------|
| bus | Communication, claims, presence | `send`, `inbox`, `claim`, `release`, `agents` |
| maw | Isolated jj workspaces | `ws create`, `ws merge`, `ws destroy` |
| br/bv | Work tracking + triage | `ready`, `create`, `close`, `--robot-next` |
| crit | Code review | `review`, `comment`, `lgtm`, `block` |
| botty | Agent runtime | `spawn`, `kill`, `tail`, `snapshot` |

### Loop Scripts

Scripts in `.agents/botbox/scripts/` automate agent loops:

| Script | Purpose |
|--------|---------|
| `agent-loop.mjs` | Worker: sequential triage-start-work-finish |
| `dev-loop.mjs` | Lead dev: triage, parallel dispatch, merge |
| `reviewer-loop.mjs` | Reviewer: review loop until queue empty |

Usage: `bun .agents/botbox/scripts/<script>.mjs <project-name> [agent-name]`
<!-- botbox:managed-end -->
