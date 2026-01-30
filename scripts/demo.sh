#!/usr/bin/env bash
# Demo script: Creates a temporary beads project with sample data for testing and screenshots

set -e

# Create temp directory
DEMO_DIR=$(mktemp -d -t beads-demo.XXXXXX)
echo "Creating demo project in: $DEMO_DIR"

# Change to demo directory
cd "$DEMO_DIR"

# Initialize beads
br init

# Create beads with various priorities and types
echo "Creating sample beads..."

# P0 Critical bug
ID1=$(br create --title "Fix critical crash on startup" --type bug --priority 0 --description="App crashes when SQLite file is missing. Need to create it on first run or show better error message." 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID1" --add-label=crash --add-label=data

# P1 High priority feature
ID2=$(br create --title "Add dark mode support" --type feature --priority 1 --description="Users are requesting a dark theme option. Should integrate with system preferences if possible." 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID2" --add-label=ui --add-label=enhancement

# P1 Important task
ID3=$(br create --title "Write user documentation" --type task --priority 1 --description="Create README with:
- Installation instructions
- Basic usage guide
- Keyboard shortcuts reference
- Configuration options" 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID3" --add-label=docs

# P2 Medium priority features
ID4=$(br create --title "Implement search functionality" --type feature --priority 2 --description="Allow filtering beads by title, description, or labels. Consider regex support." 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID4" --add-label=ui --add-label=search

ID5=$(br create --title "Add sort options" --type feature --priority 2 --description="Let users sort by priority, date, status, or title. Remember sort preference." 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID5" --add-label=ui

ID6=$(br create --title "Export to JSON/CSV" --type feature --priority 2 --description="Add export functionality for beads data in various formats." 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID6" --add-label=data --add-label=export

# P3 Lower priority
ID7=$(br create --title "Add color customization" --type feature --priority 3 --description="Allow users to customize theme colors via config file." 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID7" --add-label=ui --add-label=config

ID8=$(br create --title "Optimize database queries" --type task --priority 3 --description="Profile and optimize slow SQLite queries, especially for large bead counts." 2>&1 | grep -oE 'bd-[a-z0-9]+')
br update "$ID8" --add-label=performance --add-label=data

# Create some dependencies
echo "Adding dependencies..."
br dep add "$ID3" "$ID2" --type blocks  # Docs block dark mode (need to document it first)
br dep add "$ID4" "$ID5" --type related  # Search and sort are related features

# Add comments to some beads
echo "Adding comments..."
br comments add "$ID1" "Reproduced on Linux and macOS. Need to test on Windows too."
br comments add "$ID1" "SQLite file path should be ~/.local/share/beads/beads.db"
br comments add "$ID2" "Consider using ratatui's built-in theme support"
br comments add "$ID4" "Regex search might be overkill - start with simple substring matching"

# Close one bead to show completed work
br close "$ID8" --reason="Implemented query caching and indexed commonly-filtered columns"

# Show summary
echo ""
echo "Demo project created successfully!"
echo "Project directory: $DEMO_DIR"
echo ""
echo "Sample beads:"
br list
echo ""
echo "To explore:"
echo "  cd $DEMO_DIR"
echo "  bu"
echo ""
echo "When done, clean up with:"
echo "  rm -rf $DEMO_DIR"
