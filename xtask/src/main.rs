fn build() {
    xshell::cmd!("wasm-pack build  website --target web --out-dir ../out")
        .read()
        .unwrap();
    for entry in std::fs::read_dir("website/site").unwrap() {
        let entry = entry.unwrap();
        let dest = "out/".to_string() + entry.file_name().to_str().unwrap();
        std::fs::copy(entry.path(), dest).unwrap();
    }
}

fn serve() {
    println!("Running site at:");
    println!("http://localhost:8000");
    xshell::cmd!("python3 -m http.server --directory out/")
        .read()
        .unwrap();
}

fn github_setup() {
    xshell::cmd!("rustup target add wasm32-unknown-unknown")
        .read()
        .unwrap();
}

fn setup() {
    github_setup();
    xshell::cmd!("cargo install wasm-pack").read().unwrap();
}

fn main() {
    // Move to the root directory so commands work regardless of 'pwd'.
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let _env = xshell::pushd(path + "/..");

    let task = std::env::args().nth(1);
    match task.as_ref().map(|it| it.as_str()) {
        Some("build") => build(),
        Some("serve") => serve(),
        Some("setup") => setup(),
        Some("github_panic") => setup(),
        _ => {
            println!("Please use a command");
        }
    }
}
