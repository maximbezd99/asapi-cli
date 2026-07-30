use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const WEB_INPUTS: &[&str] = &[
    "index.html",
    "package.json",
    "package-lock.json",
    "src",
    "tsconfig.json",
    "tsconfig.app.json",
    "tsconfig.node.json",
    "vite.config.ts",
];

fn main() {
    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo must set CARGO_MANIFEST_DIR"),
    );
    let web_dir = manifest_dir.join("../../web");
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR")).join("web-dist");

    for input in WEB_INPUTS {
        println!("cargo:rerun-if-changed={}", web_dir.join(input).display());
    }

    install_dependencies_if_needed(&web_dir);
    build_web(&web_dir, &out_dir);
}

fn install_dependencies_if_needed(web_dir: &Path) {
    let package_json =
        fs::read(web_dir.join("package.json")).expect("could not read web/package.json");
    let package_lock =
        fs::read(web_dir.join("package-lock.json")).expect("could not read web/package-lock.json");
    let dependency_state = [package_json, package_lock].concat();
    let stamp = web_dir.join("node_modules/.asapi-dependencies");

    if fs::read(&stamp).ok().as_deref() == Some(dependency_state.as_slice()) {
        return;
    }

    run_npm(
        web_dir,
        &["ci", "--no-audit", "--no-fund"],
        "install web dependencies",
    );
    fs::write(stamp, dependency_state).expect("could not record installed web dependencies");
}

fn build_web(web_dir: &Path, out_dir: &Path) {
    let out_dir = out_dir
        .to_str()
        .expect("Cargo OUT_DIR must be valid UTF-8 for Vite");
    run_npm(
        web_dir,
        &["run", "build", "--", "--outDir", out_dir, "--emptyOutDir"],
        "build the web application",
    );
}

fn run_npm(web_dir: &Path, arguments: &[&str], action: &str) {
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(npm)
        .args(arguments)
        .current_dir(web_dir)
        .status()
        .unwrap_or_else(|error| {
            panic!("could not run npm to {action}; install Node.js and npm: {error}")
        });

    assert!(status.success(), "npm failed to {action} with {status}");
}
