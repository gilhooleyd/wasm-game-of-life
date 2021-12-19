all: build

build:
	wasm-pack build --target web
	cp site/* pkg/

serve:
	python3 -m http.server --directory pkg/

setup: github_setup
	cargo install wasm-pack

github_setup:
	rustup target add wasm32-unknown-unknown 
	

