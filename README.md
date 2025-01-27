# tagging thing

## Development

Quick binding for building and testing

```bash
retag() {
    PROJECT_LOCATION="$HOME/...some_dir.../retag"
    (cd "$PROJECT_LOCATION" && cargo run -- "$@")
}
```

haha actually none of this is true any more since I just added
`features = ["bundled"]` to the cargo.toml :D

~~To compile you need the SQLite development binaries:~~

Debian/Ubuntu:

```bash
sudo apt install libsqlite3-dev
```

Fedora/RHEL:

```bash
sudo dnf install sqlite-devel
```

Arch:

```bash
sudo pacman -S sqlite
```
