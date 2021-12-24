# wasm-game-of-life

This project is based on the [Rust and Webassembly](https://rustwasm.github.io/docs/book/introduction.html) project.

It has been updated to be a static site instead of a node module.

It has also been updated so there is *no* javascript, all of the code is written in rust.

The [web-sys library](https://rustwasm.github.io/wasm-bindgen/examples/dom.html) tutorials were really helpful.

## Setup

The setup installs the wasm32 rust tool chain and the wasm-pack tool.

```
cargo xtask setup
```

## Building

Building creates the website in the pkg/ directory.

```
cargo xtask build
```

## Running locally

```
cargo xtask serve
```
