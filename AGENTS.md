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

**New here?** Read [worker-loop.md](.agents/botbox/worker-loop.md) first — it covers the complete triage → start → work → finish cycle.

**All tools have `--help`** with usage examples. When unsure, run `<tool> --help` or `<tool> <command> --help`.

### Directory Structure (maw v2)

This project uses a **bare repo** layout. Source files live in workspaces under `ws/`, not at the project root.

```
project-root/          ← bare repo (no source files here)
├── ws/
│   ├── default/       ← main working copy (AGENTS.md, .beads/, src/, etc.)
│   ├── frost-castle/  ← agent workspace (isolated jj commit)
│   └── amber-reef/    ← another agent workspace
├── .jj/               ← jj repo data
├── .git/              ← git data (core.bare=true)
├── AGENTS.md          ← stub redirecting to ws/default/AGENTS.md
└── CLAUDE.md          ← symlink → AGENTS.md
```

**Key rules:**
- `ws/default/` is the main workspace — beads, config, and project files live here
- Agent workspaces (`ws/<name>/`) are isolated jj commits for concurrent work
- Use `maw exec <ws> -- <command>` to run commands in a workspace context
- Use `maw exec default -- br|bv ...` for beads commands (always in default workspace)
- Use `maw exec <ws> -- crit ...` for review commands (always in the review's workspace)
- Never run `br`, `bv`, `crit`, or `jj` directly — always go through `maw exec`

### Beads Quick Reference

| Operation | Command |
|-----------|---------|
| View ready work | `maw exec default -- br ready` |
| Show bead | `maw exec default -- br show <id>` |
| Create | `maw exec default -- br create --actor $AGENT --owner $AGENT --title="..." --type=task --priority=2` |
| Start work | `maw exec default -- br update --actor $AGENT <id> --status=in_progress --owner=$AGENT` |
| Add comment | `maw exec default -- br comments add --actor $AGENT --author $AGENT <id> "message"` |
| Close | `maw exec default -- br close --actor $AGENT <id>` |
| Add dependency | `maw exec default -- br dep add --actor $AGENT <blocked> <blocker>` |
| Sync | `maw exec default -- br sync --flush-only` |
| Triage (scores) | `maw exec default -- bv --robot-triage` |
| Next bead | `maw exec default -- bv --robot-next` |

**Required flags**: `--actor $AGENT` on mutations, `--author $AGENT` on comments.

### Workspace Quick Reference

| Operation | Command |
|-----------|---------|
| Create workspace | `maw ws create <name>` |
| List workspaces | `maw ws list` |
| Merge to main | `maw ws merge <name> --destroy` |
| Destroy (no merge) | `maw ws destroy <name>` |
| Run jj in workspace | `maw exec <name> -- jj <jj-args...>` |

**Avoiding divergent commits**: Each workspace owns ONE commit. Only modify your own.

| Safe | Dangerous |
|------|-----------|
| `jj describe` (your working copy) | `jj describe main -m "..."` |
| `maw exec <your-ws> -- jj describe -m "..."` | `jj describe <other-change-id>` |

If you see `(divergent)` in `jj log`:
```bash
jj abandon <change-id>/0   # keep one, abandon the divergent copy
```

### Beads Conventions

- Create a bead before starting work. Update status: `open` → `in_progress` → `closed`.
- Post progress comments during work for crash recovery.
- **Push to main** after completing beads (see [finish.md](.agents/botbox/finish.md)).
- **Install locally** after releasing: `just install`

### Identity

Your agent name is set by the hook or script that launched you. Use `$AGENT` in commands.
For manual sessions, use `<project>-dev` (e.g., `myapp-dev`).

### Claims

When working on a bead, stake claims to prevent conflicts:

```bash
bus claims stake --agent $AGENT "bead://<project>/<id>" -m "<id>"
bus claims stake --agent $AGENT "workspace://<project>/<ws>" -m "<id>"
bus claims release --agent $AGENT --all  # when done
```

### Reviews

Use `@<project>-<role>` mentions to request reviews:

```bash
maw exec $WS -- crit reviews request <review-id> --reviewers $PROJECT-security --agent $AGENT
bus send --agent $AGENT $PROJECT "Review requested: <review-id> @$PROJECT-security" -L review-request
```

The @mention triggers the auto-spawn hook for the reviewer.

### Cross-Project Communication

**Don't suffer in silence.** If a tool confuses you or behaves unexpectedly, post to its project channel.

1. Find the project: `bus history projects -n 50` (the #projects channel has project registry entries)
2. Post question or feedback: `bus send --agent $AGENT <project> "..." -L feedback`
3. For bugs, create beads in their repo first
4. **Always create a local tracking bead** so you check back later:
   ```bash
   maw exec default -- br create --actor $AGENT --owner $AGENT --title="[tracking] <summary>" --labels tracking --type=task --priority=3
   ```

See [cross-channel.md](.agents/botbox/cross-channel.md) for the full workflow.

### Session Search (optional)

Use `cass search "error or problem"` to find how similar issues were solved in past sessions.


### Design Guidelines

- [CLI tool design for humans, agents, and machines](.agents/botbox/design/cli-conventions.md)

### Workflow Docs

- [Ask questions, report bugs, and track responses across projects](.agents/botbox/cross-channel.md)
- [Close bead, merge workspace, release claims, sync](.agents/botbox/finish.md)
- [groom](.agents/botbox/groom.md)
- [Verify approval before merge](.agents/botbox/merge-check.md)
- [Turn specs/PRDs into actionable beads](.agents/botbox/planning.md)
- [Validate toolchain health](.agents/botbox/preflight.md)
- [Create and validate proposals before implementation](.agents/botbox/proposal.md)
- [Report bugs/features to other projects](.agents/botbox/report-issue.md)
- [Reviewer agent loop](.agents/botbox/review-loop.md)
- [Request a review](.agents/botbox/review-request.md)
- [Handle reviewer feedback (fix/address/defer)](.agents/botbox/review-response.md)
- [Explore unfamiliar code before planning](.agents/botbox/scout.md)
- [Claim bead, create workspace, announce](.agents/botbox/start.md)
- [Find work from inbox and beads](.agents/botbox/triage.md)
- [Change bead status (open/in_progress/blocked/done)](.agents/botbox/update.md)
- [Full triage-work-finish lifecycle](.agents/botbox/worker-loop.md)
<!-- botbox:managed-end -->
