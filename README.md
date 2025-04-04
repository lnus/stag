<p align="center">
    <img width="300" src="https://c.tenor.com/r5c67WCHZZcAAAAC/tenor.gif" alt="Are you ready to become a stag">
    <h1 align="center">stag 🦌</h1>
</p>
<p align="center">
    (S)Tag Management Tool | Very work in progress 🚧
</p>
<p align="center">
    <a href="https://github.com/lnus/stag/actions/workflows/rust.yml">
        <img src="https://github.com/lnus/stag/actions/workflows/rust.yml/badge.svg" alt="Rust CI">
    </a> <!-- look ma, CI! -->
</p>

Q: What does the **S** stand for?

A: Semantic, Storage, Sorting, System, Stag (recursive)

## TODO's and scope creep 🛠️

- [ ] feat: Directory watching using inotify
- [ ] feat: Add a validate/clean command for broken tag-links
- [ ] feat/fix: Add config from `$XDG_CONFIG_HOME`
- [ ] fix: Validate tag names and CaSeS of them
- [ ] fix: Fix recursion through symlinks
- [x] feat: Add negations in search
- [x] feat: Add metadata based autotagging (filetype, size, etc...)
- [x] feat: Extend metadata based autotagging
- [x] feat: A display of all current active tags
- [x] perf: Make negations in search faster, this is slow for large searches
- [x] perf: Fix a bunch of the SQL queries in general
- [x] refactor: Clean up the SQL queries, they are a pain to read
- [x] refactor: Split up stuff more cleanly

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

## Usage

### Basic Tag Management

```bash
# Tag directories
stag a proj ~/Projects/* # All/* now has the tag proj
stag a rust ~/Projects/my-rust-project # This now has the tags proj && rust

# Recursively tag all files in directory
stag a rust ~/Projects/my-rust-project -r # All files in my-rust-project now have the tag rust

# Recursively tag hidden files (dotfiles, gitignore/ignored files)
# TODO: This behaviour needs better documentation
stag a config ~/.config -r --hidden # Will tag files that are ignored by default

# Remove tags (same as above applies, in reverse)
stag rm rust ~/Projects/old-project
stag rm docs ~/Projects/*/docs -r
stag rm config ~/.config -r --hidden
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

# Find projects that are NOT rust
stag s proj -e rust

# Find anything tagged either rust or docs (OR search)
stag s rust docs --any

# List everything with a specific tag
stag ls docs
stag ls docs --dirs   # Only directories
stag ls docs --files  # Only files
```

### Autotagging

```bash
# Stag allows for basic metadata autotagging
stag at README.md

# Inspecting this gives
stag i README.md # small, mime:text/markdown, text, file, x-markdown, markdown, mime:text/x-markdown
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

# Copy all directories into a new location for backup
stag s proj --dirs | xargs -I {} cp {} . -r

# Check git status across multiple projects
stag s proj --dirs | xargs -I{} sh -c 'echo "=== {} ===" && cd {} && git status'

# Format all Rust projects
stag s proj rust --dirs | xargs -I{} cargo fmt --manifest-path {}/Cargo.toml

# Test all Rust projects
stag s proj rust --dirs | xargs -I{} cargo test --manifest-path {}/Cargo.toml
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
scd                    # Navigate tagged projects
scd "rust wip"         # Navigate WIP Rust projects
```

### Tips

- Tags stored in standard XDG path (~/.local/share/stag/tags.db)
- Tags are flat (no hierarchy) but you can create your own conventions like project/frontend
- Use with xargs for powerful batch operations
- Combine with fzf for interactive filtering
- Directory tagging is default, use -r for recursive file tagging
- Searches use AND by default, use --any for OR operation
