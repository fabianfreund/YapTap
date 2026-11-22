fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        // CPAL (handy-recorder, rodio) links CoreAudio functions that only exist
        // on macOS 14.2+. Weak-link the framework so YapTap launches on 13.
        println!("cargo:rustc-link-arg=-Wl,-weak_framework,CoreAudio");
        link_clang_rt_osx();
    }
    tauri_build::build()
}

/// Links clang's compiler runtime, which provides `__isPlatformVersionAtLeast`.
///
/// ggml-metal guards newer Metal APIs with `@available`. With a deployment
/// target older than the SDK (macOS 13 here), clang lowers those checks to
/// calls into `libclang_rt.osx.a`, which rustc doesn't link by default.
/// Same fix as Handy's build.rs (MIT).
fn link_clang_rt_osx() {
    let output = std::process::Command::new("clang")
        .arg("--print-resource-dir")
        .output()
        .expect("run `clang --print-resource-dir` to locate libclang_rt.osx.a");
    assert!(output.status.success(), "`clang --print-resource-dir` failed");
    let resource_dir = String::from_utf8(output.stdout).expect("clang resource dir is UTF-8");
    let darwin_dir = std::path::Path::new(resource_dir.trim()).join("lib/darwin");
    assert!(
        darwin_dir.join("libclang_rt.osx.a").exists(),
        "libclang_rt.osx.a not found in {}",
        darwin_dir.display()
    );
    println!("cargo:rustc-link-search=native={}", darwin_dir.display());
    println!("cargo:rustc-link-lib=static=clang_rt.osx");
}
