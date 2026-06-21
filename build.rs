use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let host = env::var("HOST").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target == host {
        setup_tools(&host);
    }

    if target_os == "android" {
        setup_android(&target_arch);
    }
}


fn setup_tools(host: &str) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    cargo_install(&cargo, "cargo-ndk", &["install", "cargo-ndk", "--locked"]);
    cargo_install(
        &cargo,
        "cargo-zigbuild",
        &["install", "cargo-zigbuild", "--locked"],
    );
    cargo_install(&cargo, "cargo-xwin", &["install", "cargo-xwin", "--locked"]);
    cargo_install(
        &cargo,
        "gobley-uniffi-bindgen",
        &[
            "install",
            "gobley-uniffi-bindgen",
            "--git",
            "https://github.com/gobley/gobley",
            "--tag",
            "v0.3.4",
        ],
    );

    ensure_zig();
    // Cross-compilation targets available on any host
    rustup_target_add(&[
        // Android
        "aarch64-linux-android",
        "armv7-linux-androideabi",
        "i686-linux-android",
        "x86_64-linux-android",
        // Linux glibc + musl + FreeBSD
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "armv7-unknown-linux-gnueabihf",
        "i686-unknown-linux-gnu",
        "riscv64gc-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
        "armv7-unknown-linux-musleabihf",
        "i686-unknown-linux-musl",
        "riscv64gc-unknown-linux-musl",
        "x86_64-unknown-freebsd",
        // Windows MSVC
        "x86_64-pc-windows-msvc",
        "i686-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
    ]);

    // Apple targets are only available on macOS hosts
    if host.contains("apple") {
        rustup_target_add(&[
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "aarch64-apple-ios",
            "aarch64-apple-ios-sim",
            "x86_64-apple-ios",
        ]);
    }
}

fn cargo_install(cargo: &str, binary: &str, args: &[&str]) {
    if Command::new(binary).arg("--version").output().is_ok() {
        return;
    }
    println!("cargo:warning=installing {binary} ...");
    let status = Command::new(cargo)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to launch `cargo install` for {binary}: {e}"));
    assert!(status.success(), "`cargo install` for {binary} failed");
}

fn rustup_target_add(targets: &[&str]) {
    let mut args = vec!["target", "add"];
    args.extend_from_slice(targets);
    let status = Command::new("rustup")
        .args(&args)
        .status()
        .unwrap_or_else(|e| panic!("failed to run rustup: {e}"));
    assert!(status.success(), "rustup target add failed");
}

/// Check that `zig` is available; it cannot be installed via `cargo install`.
fn ensure_zig() {
    if Command::new("zig").arg("version").output().is_ok() {
        return;
    }
    println!(
        "cargo:warning=`zig` not found in PATH — install it from \
         https://ziglang.org/download/ or via your package manager \
         before running targets that use cargo-zigbuild."
    );
}

// ── Platform-specific compilation setup ──────────────────────────────────────

/// Android-specific build configuration.
///
/// All Android targets need the 16 KiB page-size flag so the .so is accepted
/// by Android 15+ devices that use 16 KiB pages.
///
/// x86_64 additionally requires a workaround for rust-lang/rust#109717 where
/// `libclang_rt.builtins` is not statically linked by default, causing missing
/// symbol errors at runtime on some NDK versions.
/// Workaround from https://github.com/mozilla/application-services/pull/5442,
/// inspired by https://github.com/rust-nostr/nostr-sdk-ffi/blob/master/build.rs
fn setup_android(arch: &str) {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-z,max-page-size=16384");

    if arch == "x86_64" {
        // CC_x86_64-linux-android is only set when cargo-ndk is driving the
        // build. Skip the workaround when building the target another way.
        let Ok(cc) = env::var("CC_x86_64-linux-android") else {
            return;
        };
        let clang_path = PathBuf::from(cc);
        let toolchain_path = clang_path
            .ancestors()
            .nth(2)
            .expect("could not resolve NDK toolchain path from CC_x86_64-linux-android")
            .to_str()
            .expect("NDK toolchain path is not valid UTF-8");
        let ver = clang_major_version(&clang_path);
        println!("cargo:rustc-link-search={toolchain_path}/lib/clang/{ver}/lib/linux/");
        println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
    }
}

fn clang_major_version(clang: &Path) -> String {
    let out = Command::new(clang)
        .arg("-dumpversion")
        .output()
        .expect("failed to run clang");
    if !out.status.success() {
        panic!(
            "clang -dumpversion failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    String::from_utf8(out.stdout)
        .expect("clang output is not UTF-8")
        .split('.')
        .next()
        .expect("unexpected clang version format")
        .trim()
        .to_owned()
}