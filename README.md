<h1 align="center">
    stag 🦌 | WIP 🏗️
</h1>
<p align="center">
    (S)Tag Management Tool
</p>
<p align="center">
    <img src="https://c.tenor.com/r5c67WCHZZcAAAAC/tenor.gif" alt="Are you ready to become a stag"> 
</p>
<p align="center">
    <strong>⚠️ SECRET FRIENDS AND FAMILY EDITION ⚠️</strong>
</p>

Q: What does the **S** stand for?

A:
- Super
- Sorting
- Storage
- System
- Stag (recursive)
- Simpsons
- Something else entirely

## Installation

1. Clone this repository.
2. Run `cargo install --path .`
3. Check if `which stag` works
4. If yes, good to go baby 😎

```bash
cargo install --path .
```

### Requirements

- Rust Version: 🤔 I'm running `rustc 1.83.0`, so that or higher I guess.
    - I'm 99% sure any 2021 version should work.
- Just get [rustup](https://rustup.rs/) and install latest.

## TODO's and scope creep 🛠️

- [x] feat: Add negations in search
- [ ] feat: Add a validate/clean command for broken tag-links
- [ ] feat: Add metadata based autotagging (filetype, size, etc...)
- [ ] feat: Some directory wathing, this is a huge *maybe* 
- [ ] feat/fix: Add config from `$XDG_CONFIG_HOME`
- [ ] perf: Make negations in search faster, this is slow for large searches
- [ ] perf: Fix a bunch of the SQL queries in general
- [ ] perf: Do all of the filtering directly in the SQL, rather than after (should be faster?)
- [ ] refactor: Clean up the SQL queries, they are a pain to read
- [ ] refactor: Split up stuff more cleanly (if needed)

## Usage

### Basic Tag Management

```bash
# Tag directories
stag a proj ~/Projects/* # All/* now has the tag proj
stag a rust ~/Projects/my-rust-project # This now has the tags proj && rust

# Recursively tag all files in directory
stag a -r rust ~/Projects/my-rust-project # All files in my-rust-project now have the tag rust

# Remove tags (same as above applies, in reverse)
stag rm rust ~/Projects/old-project
stag rm -r docs ~/Projects/*/docs
```

### Searching and Filtering

```bash
# Find all projects
stag s proj

# Find only directories or files
stag s proj --dirs    # Only show directories
stag s proj --files   # Only show files

# Find Rust projects
stag s proj rust

# Find projects that NOT rust
stag s proj -e rust

# Find anything tagged either rust or docs (OR search)
stag s rust docs --any

# List everything with a specific tag
stag ls docs
stag ls docs --dirs   # Only directories
stag ls docs --files  # Only files
```

### Shell Integration

```bash
# Quick project navigation function
scd() {
    local tags="${1:-proj}"  # Default to 'proj' if no args
    local dir=$(stag s $tags --dirs | fzf)
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
stag s proj --dirs | xargs du -sh | sort -hr
# Output:
# 1.2G    ~/Projects/big-data-project
# 856M    ~/Projects/web-app
# 234M    ~/Projects/rust-game

# Find large files, just in general!
stag s images --files | xargs du -sh | sort -hr
# Output:
# 221G    ~/Images/yourmom.png
# 1.2G    ~/Images/react-logo.svg
# 8.0K    ~/Images/mymom.webp

# Check git status across multiple projects
stag s proj --dirs | xargs -I{} sh -c 'echo "=== {} ===" && cd {} && git status'
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
stag s proj rust --dirs | xargs -I{} cargo fmt --manifest-path {}/Cargo.toml

# Test all Rust projects
stag s proj rust --dirs | xargs -I{} cargo test --manifest-path {}/Cargo.toml
```

### Tips

- Tags stored in standard XDG path (~/.local/share/stag/tags.db)
- Tags are flat (no hierarchy) but you can create your own conventions like project/frontend
- Use with xargs for powerful batch operations
- Combine with fzf for interactive filtering
- Directory tagging is default, use -r for recursive file tagging
- Searches use AND by default, use --any for OR operation
