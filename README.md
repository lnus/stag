# ReTag (Name is a work in progress lmao)

## Installation

For now just `git clone https://github.com/lnus/retag.git` and do:

```bash
cargo install --path .
```

*It's a private repo for god sake*

## Usage

### Basic Tag Management

```bash
# Tag directories
retag a proj ~/Projects/*
retag a rust ~/Projects/my-rust-project
retag a wip ~/Projects/in-progress
retag a docs ~/Projects/*/docs

# Tag with multiple tags
retag a rust ~/Projects/cool-project
retag a wip ~/Projects/cool-project

# Recursively tag all files in directory
retag a -r rust ~/Projects/my-rust-project
retag a -r rust ~/Projects/my-rust-project/src/*.rs

# Remove tags
retag rm rust ~/Projects/old-project
retag rm -r docs ~/Projects/*/docs
```

### Searching and Filtering

```bash
# Find all projects
retag s proj

# Find only directories or files
retag s proj --dirs    # Only show directories
retag s proj --files   # Only show files

# Find Rust projects
retag s proj rust

# Find Rust projects that are work-in-progress
retag s proj rust wip

# Find anything tagged either rust or docs (OR search)
retag s rust docs --any

# List everything with a specific tag
retag ls docs
retag ls docs --dirs   # Only directories
retag ls docs --files  # Only files
```

### Shell Integration

```bash
# Quick project navigation function
rcd() {
    local tags="${1:-proj}"  # Default to 'proj' if no args
    local dir=$(retag s $tags --dirs | fzf)
    if [ -n "$dir" ]; then
        cd "$dir"
    fi
}

# Usage:
rcd                    # Navigate tagged projects
rcd "rust wip"         # Navigate WIP Rust projects
```

### Combining with Unix Tools

```bash
# Find large files in tagged projects
retag s proj --dirs | xargs du -sh | sort -hr
# Output:
# 1.2G    ~/Projects/big-data-project
# 856M    ~/Projects/web-app
# 234M    ~/Projects/rust-game

# Find large files, just in general!
retag s images --files | xargs du -sh | sort -hr
# Output:
# 221G    ~/Images/yourmom.png
# 1.2G    ~/Images/react-logo.svg
# 8.0K    ~/Images/mymom.webp

# Check git status across multiple projects
retag s proj --dirs | xargs -I{} sh -c 'echo "=== {} ===" && cd {} && git status'
# Output:
# === ~/Projects/cli-tools ===
# On branch main
# Changes not staged for commit:
#   modified:   src/main.rs
#
# === ~/Projects/web-app ===
# On branch feature/auth
# Your branch is up to date with 'origin/feature/auth'
# nothing to commit, working tree clean


# Format all Rust projects
retag s proj rust --dirs | xargs -I{} cargo fmt --manifest-path {}/Cargo.toml

# Test all Rust projects
retag s proj rust --dirs | xargs -I{} cargo test --manifest-path {}/Cargo.toml
```

### Tips

- Tags stored in standard XDG path (~/.local/share/retag/tags.db)
- Tags are flat (no hierarchy) but you can create your own conventions like project/frontend
- Use with xargs for powerful batch operations
- Combine with fzf for interactive filtering
- Directory tagging is default, use -r for recursive file tagging
- Searches use AND by default, use --any for OR operation

## Development

Quick binding for building and testing

```bash
cargo install --path .
```
