# retag - A Tag Management Tool (WIP 🏗️)

**⚠️ SECRET FRIENDS AND FAMILY EDITION ⚠️** 

![anime girl dancing](https://i.giphy.com/11lxCeKo6cHkJy.webp) 

## Name suggestions

Oh my god please give me name suggestions

### 🐧 inspo

1. stag (S-tag)
    - It sounds really cool
    - Stag/deer iconography could be used, wow
    - It literally has tag in the name
    - It's not me trying to be japanese and cringe
    - Memorable
    - Feels super good to write, left hand only touch type
    - I think this is the one I'm going for

1. tagr
    - Terse, memorable
    - Follows Unix tool naming conventions
    - Easy to type
    - Clear purpose

2. tg
    - Ultra-minimal
    - Very easy to type
    - Might be too generic/clash with other tools

### 日本語 inspo

1. タグ (tagu) 
    - means tag, literally, note or tag
    - ehhhh? kinda cringu　キモイ core if you ask me

2. 整理 / せいり / セイリ (seiri, katakana for style points) 
    - is better, more obscurish, means organization
    - organization/tagging/sorting, pretty good vibes in general
    - semi annoying to type
    - could alias the binary to sei/sri etc..?
    - con: surprisingly also means menstruation 生理, huh. who knew.

3. 分類 / ぶんるい / ブンルイ (bunrui)
    - literally means classification
    - sounds kinda cool
    - annoying to type

## Installation

1. Clone this repository.
2. Run `cargo install --path .`
3. Check if `which retag` works
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
