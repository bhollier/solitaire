# Solitaire

[![CI](https://github.com/bhollier/solitaire/actions/workflows/rust.yml/badge.svg)](https://github.com/bhollier/solitaire/actions/workflows/rust.yml)

Solitaire in rust 🦀

Playable demo coming soon!

### Terminal UI

![demo](./demo/demo.gif)

Powered by [ratatui](https://github.com/ratatui-org/ratatui).

Run it with:

```bash
git clone https://github.com/bhollier/solitaire
cd solitiare
cargo run --package solitaire --bin solitaire-tui --features=tui
```

Or to play in a browser:

```bash
cargo install --locked trunk
rustup target add wasm32-unknown-unknown
trunk serve
```