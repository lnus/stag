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

# Find Rust projects
retag s proj rust

# Find Rust projects that are work-in-progress
retag s proj rust wip

# Find anything tagged either rust or docs (OR search)
retag s rust docs --any

# List everything with a specific tag
retag ls docs
```

### Combining with Unix Tools

```bash
# Interactive project selection with fzf
cd $(retag s proj | fzf)

# Find large files in tagged projects
retag s proj | xargs du -sh | sort -hr

# Check git status across multiple projects
retag s proj | xargs -I{} sh -c 'echo "=== {} ===" && cd {} && git status'

# Format all Rust projects
retag s proj rust | xargs -I{} cargo fmt --manifest-path {}/Cargo.toml
```

### Tips

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
