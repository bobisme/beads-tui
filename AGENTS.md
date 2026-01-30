# Agent Instructions

This document provides guidance for AI coding agents working on `beads-tui` (`bu`).

## Project Overview

`bu` is a Rust/Ratatui TUI for viewing and managing beads (issues). It reads from SQLite (`.beads/beads.db`) and shells out to `br` (beads_rust) for mutations.

## Quick Reference

```bash
# Build and run
cargo build
cargo run

# Run tests
cargo test

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
cargo test

# Bump version in Cargo.toml
# e.g., 0.3.1 to 0.4.0

# Commit
git add -A
git commit -m "feat(scope): description

Co-Authored-By: Claude <noreply@anthropic.com>"

# Tag the release and push
git tag vX.Y.Z
git push && git push origin vX.Y.Z

# Install locally
just install

# Verify
bu --version

# Announce on botbus
botbus --agent <your-agent> send beads-tui "Released vX.Y.Z - [summary of changes]"
```

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
- Run `cargo test` before committing

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

<!-- crit-agent-instructions -->

## Crit: Agent-Centric Code Review

This project uses [crit](https://github.com/anomalyco/botcrit) for distributed code reviews optimized for AI agents.

### Quick Start

```bash
# Initialize crit in the repository (once)
crit init

# Create a review for current change
crit reviews create --title "Add feature X"

# List open reviews
crit reviews list

# Check reviews needing your attention
crit reviews list --needs-review --author $BOTBUS_AGENT

# Show review details
crit reviews show <review_id>
```

### Adding Comments (Recommended)

The simplest way to comment on code - auto-creates threads:

```bash
# Add a comment on a specific line (creates thread automatically)
crit comment <review_id> --file src/main.rs --line 42 "Consider using Option here"

# Add another comment on same line (reuses existing thread)
crit comment <review_id> --file src/main.rs --line 42 "Good point, will fix"

# Comment on a line range
crit comment <review_id> --file src/main.rs --line 10-20 "This block needs refactoring"
```

### Managing Threads

```bash
# List threads on a review
crit threads list <review_id>

# Show thread with context
crit threads show <thread_id>

# Resolve a thread
crit threads resolve <thread_id> --reason "Fixed in latest commit"
```

### Voting on Reviews

```bash
# Approve a review (LGTM)
crit lgtm <review_id> -m "Looks good!"

# Block a review (request changes)
crit block <review_id> -r "Need more test coverage"
```

### Viewing Full Reviews

```bash
# Show full review with all threads and comments
crit review <review_id>

# Show with more context lines
crit review <review_id> --context 5

# List threads with first comment preview
crit threads list <review_id> -v
```

### Approving and Merging

```bash
# Approve a review (changes status to approved)
crit reviews approve <review_id>

# Mark as merged (after jj squash/merge)
# Note: Will fail if there are blocking votes
crit reviews merge <review_id>

# Self-approve and merge in one step (solo workflows)
crit reviews merge <review_id> --self-approve
```

### Agent Best Practices

1. **Set your identity** via environment:
   ```bash
   export BOTBUS_AGENT=my-agent-name
   ```

2. **Check for pending reviews** at session start:
   ```bash
   crit reviews list --needs-review --author $BOTBUS_AGENT
   ```

3. **Check status** to see unresolved threads:
   ```bash
   crit status <review_id> --unresolved-only
   ```

4. **Run doctor** to verify setup:
   ```bash
   crit doctor
   ```

### Output Formats

- Default output is TOON (token-optimized, human-readable)
- Use `--json` flag for machine-parseable JSON output

### Key Concepts

- **Reviews** are anchored to jj Change IDs (survive rebases)
- **Threads** group comments on specific file locations
- **crit comment** is the simple way to leave feedback (auto-creates threads)
- Works across jj workspaces (shared .crit/ in main repo)

<!-- end-crit-agent-instructions -->