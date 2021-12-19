# wasm-game-of-life

This project is based on the [Rust and Webassembly](https://rustwasm.github.io/docs/book/introduction.html) project.

It has been updated to be a static site instead of a node module.

It has also been updated so there is *no* javascript, all of the code is written in rust.

The [web-sys library](https://rustwasm.github.io/wasm-bindgen/examples/dom.html) tutorials were really helpful.

## Building

```
wasm-pack build --target web
```

## Running locally

```
python -m SimpleHTTPServer
```
