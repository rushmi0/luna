set shell := ["bash", "-euo", "pipefail", "-c"]

export PATH := env_var("HOME") / ".cargo/bin" + ":" + env_var("PATH")

script_dir  := justfile_directory()
target_dir  := script_dir / "target"
config      := script_dir / "uniffi-kmp.toml"
src_dir     := script_dir / "libkmp/lau-kmp/src"
android_dir := src_dir / "androidMain"
jni_dir     := android_dir / "jniLibs"
common_dir  := src_dir / "commonMain"
native_dir  := src_dir / "nativeMain"
jvm_dir     := src_dir / "jvmMain"
artifact    := "luna"

# Full pipeline (add ios before clean-kmp when building on macOS)
default: build-host build-android build-linux build-windows clean-kmp generate-bindings copy-android copy-linux-glibc copy-linux-musl copy-freebsd copy-windows build-kmp

# Build and copy Android libs
android: build-android copy-android

# Build and copy iOS static libs (macOS only)
ios: build-ios copy-ios

# Build and copy Linux + FreeBSD JVM resources
linux: build-linux copy-linux-glibc copy-linux-musl copy-freebsd

# Build and copy all JVM resources (Linux + Windows; add macos on macOS)
jvm: linux build-windows copy-windows


setup:
    cargo check

# Host Linux x86_64 — used by gobley-uniffi-bindgen
build-host: setup
    cargo build --lib --release

build-android: setup
    cargo ndk -t arm64-v8a -t armeabi-v7a -t x86 -t x86_64 -P 24 build --lib --release

# macOS only
build-ios: setup
    #!/usr/bin/env bash
    set -euxo pipefail
    for TARGET in \
        aarch64-apple-ios     \
        aarch64-apple-ios-sim \
        x86_64-apple-ios; do
        cargo zigbuild --lib --release --target "$TARGET"
    done

build-linux: setup
    #!/usr/bin/env bash
    set -euxo pipefail
    for TARGET in \
        x86_64-unknown-linux-gnu       \
        aarch64-unknown-linux-gnu      \
        armv7-unknown-linux-gnueabihf  \
        i686-unknown-linux-gnu         \
        riscv64gc-unknown-linux-gnu    \
        x86_64-unknown-linux-musl      \
        aarch64-unknown-linux-musl     \
        armv7-unknown-linux-musleabihf \
        i686-unknown-linux-musl        \
        riscv64gc-unknown-linux-musl   \
        x86_64-unknown-freebsd; do
        cargo zigbuild --lib --release --target "$TARGET"
    done

# XWIN_ARCH must include x86; default is only x86_64,aarch64
build-windows: setup
    #!/usr/bin/env bash
    set -euxo pipefail
    export LUNA_REAL_CLANG="$(PATH=/usr/local/bin:/usr/bin:/bin command -v clang)"
    export PATH="{{script_dir}}/scripts:$PATH"
    export CLANG_CL=clang-cl
    for TARGET in \
        x86_64-pc-windows-msvc  \
        i686-pc-windows-msvc    \
        aarch64-pc-windows-msvc; do
        XWIN_ARCH=x86,x86_64,aarch64 cargo xwin build --lib --release --target "$TARGET"
    done

# macOS only
build-macos: setup
    #!/usr/bin/env bash
    set -euxo pipefail
    for TARGET in aarch64-apple-darwin x86_64-apple-darwin; do
        cargo zigbuild --lib --release --target "$TARGET"
    done

clean-kmp:
    rm -rf "{{android_dir}}" "{{common_dir}}" "{{native_dir}}" "{{jvm_dir}}" \
           "{{src_dir}}/nativeInterop/cinterop/headers"

clean: clean-kmp
    rm -rf "{{target_dir}}" \
           "{{script_dir}}/libkmp/lau-kmp/build" \
           "{{script_dir}}/libkmp/build"

copy-android:
    mkdir -p "{{jni_dir}}/arm64-v8a" "{{jni_dir}}/armeabi-v7a" "{{jni_dir}}/x86" "{{jni_dir}}/x86_64"
    cp "{{target_dir}}/aarch64-linux-android/release/lib{{artifact}}.so"   "{{jni_dir}}/arm64-v8a/"
    cp "{{target_dir}}/armv7-linux-androideabi/release/lib{{artifact}}.so" "{{jni_dir}}/armeabi-v7a/"
    cp "{{target_dir}}/i686-linux-android/release/lib{{artifact}}.so"      "{{jni_dir}}/x86/"
    cp "{{target_dir}}/x86_64-linux-android/release/lib{{artifact}}.so"    "{{jni_dir}}/x86_64/"

# macOS only
copy-ios:
    mkdir -p "{{src_dir}}/lib/ios-arm64" "{{src_dir}}/lib/ios-simulator-arm64" "{{src_dir}}/lib/ios-simulator-x64"
    cp "{{target_dir}}/aarch64-apple-ios/release/lib{{artifact}}.a"     "{{src_dir}}/lib/ios-arm64/"
    cp "{{target_dir}}/aarch64-apple-ios-sim/release/lib{{artifact}}.a" "{{src_dir}}/lib/ios-simulator-arm64/"
    cp "{{target_dir}}/x86_64-apple-ios/release/lib{{artifact}}.a"      "{{src_dir}}/lib/ios-simulator-x64/"

copy-linux-glibc:
    mkdir -p "{{jvm_dir}}/resources/linux-x86"   "{{jvm_dir}}/resources/linux-x86-64" \
             "{{jvm_dir}}/resources/linux-arm"    "{{jvm_dir}}/resources/linux-aarch64" \
             "{{jvm_dir}}/resources/linux-riscv64"
    cp "{{target_dir}}/i686-unknown-linux-gnu/release/lib{{artifact}}.so"        "{{jvm_dir}}/resources/linux-x86/"
    cp "{{target_dir}}/x86_64-unknown-linux-gnu/release/lib{{artifact}}.so"      "{{jvm_dir}}/resources/linux-x86-64/"
    cp "{{target_dir}}/armv7-unknown-linux-gnueabihf/release/lib{{artifact}}.so" "{{jvm_dir}}/resources/linux-arm/"
    cp "{{target_dir}}/aarch64-unknown-linux-gnu/release/lib{{artifact}}.so"     "{{jvm_dir}}/resources/linux-aarch64/"
    cp "{{target_dir}}/riscv64gc-unknown-linux-gnu/release/lib{{artifact}}.so"   "{{jvm_dir}}/resources/linux-riscv64/"

copy-linux-musl:
    mkdir -p "{{jvm_dir}}/resources/linux-x86-musl"   "{{jvm_dir}}/resources/linux-x86-64-musl" \
             "{{jvm_dir}}/resources/linux-arm-musl"    "{{jvm_dir}}/resources/linux-aarch64-musl" \
             "{{jvm_dir}}/resources/linux-riscv64-musl"
    cp "{{target_dir}}/i686-unknown-linux-musl/release/lib{{artifact}}.so"        "{{jvm_dir}}/resources/linux-x86-musl/"
    cp "{{target_dir}}/x86_64-unknown-linux-musl/release/lib{{artifact}}.so"      "{{jvm_dir}}/resources/linux-x86-64-musl/"
    cp "{{target_dir}}/armv7-unknown-linux-musleabihf/release/lib{{artifact}}.so" "{{jvm_dir}}/resources/linux-arm-musl/"
    cp "{{target_dir}}/aarch64-unknown-linux-musl/release/lib{{artifact}}.so"     "{{jvm_dir}}/resources/linux-aarch64-musl/"
    cp "{{target_dir}}/riscv64gc-unknown-linux-musl/release/lib{{artifact}}.so"   "{{jvm_dir}}/resources/linux-riscv64-musl/"

copy-freebsd:
    mkdir -p "{{jvm_dir}}/resources/freebsd-x86-64"
    cp "{{target_dir}}/x86_64-unknown-freebsd/release/lib{{artifact}}.so" "{{jvm_dir}}/resources/freebsd-x86-64/"

copy-windows:
    mkdir -p "{{jvm_dir}}/resources/win32-x86" "{{jvm_dir}}/resources/win32-x86-64" "{{jvm_dir}}/resources/win32-aarch64"
    cp "{{target_dir}}/i686-pc-windows-msvc/release/{{artifact}}.dll"    "{{jvm_dir}}/resources/win32-x86/"
    cp "{{target_dir}}/x86_64-pc-windows-msvc/release/{{artifact}}.dll"  "{{jvm_dir}}/resources/win32-x86-64/"
    cp "{{target_dir}}/aarch64-pc-windows-msvc/release/{{artifact}}.dll" "{{jvm_dir}}/resources/win32-aarch64/"

# macOS only
copy-macos:
     mkdir -p "{{jvm_dir}}/resources/darwin-x86-64" "{{jvm_dir}}/resources/darwin-aarch64"
     cp "{{target_dir}}/x86_64-apple-darwin/release/lib{{artifact}}.dylib"  "{{jvm_dir}}/resources/darwin-x86-64/"
     cp "{{target_dir}}/aarch64-apple-darwin/release/lib{{artifact}}.dylib" "{{jvm_dir}}/resources/darwin-aarch64/"


# Build unstripped debug lib (release profile has strip=true), generate KMP bindings, copy to src
generate-bindings: setup
    cargo build --lib --target x86_64-unknown-linux-gnu
    gobley-uniffi-bindgen \
        --library "{{target_dir}}/x86_64-unknown-linux-gnu/debug/lib{{artifact}}.so" \
        --config  "{{config}}" \
        --out-dir "{{target_dir}}/uniffi/kotlin-multiplatform"
    ls -l "{{target_dir}}/uniffi/kotlin-multiplatform/"
    mkdir -p "{{src_dir}}"
    cp -R "{{target_dir}}/uniffi/kotlin-multiplatform/." "{{src_dir}}"

build-kmp:
    cd "{{script_dir}}/libkmp" && ./gradlew :lau-kmp:assemble

# Run all lau-kmp test that can execute on the current host
test-kmp: test-kmp-jvm test-kmp-android

# Run lau-kmp JVM test
test-kmp-jvm:
    cd "{{script_dir}}/libkmp" && ./gradlew :lau-kmp:jvmTest

# Run lau-kmp Android host test (runs on JVM, no device needed)
test-kmp-android:
    cd "{{script_dir}}/libkmp" && ./gradlew :lau-kmp:testAndroidHostTest

# Run lau-kmp iOS simulator test (macOS only)
test-kmp-ios:
    cd "{{script_dir}}/libkmp" && ./gradlew :lau-kmp:iosSimulatorArm64Test

# Run every lau-kmp test including iOS (macOS only)
test-kmp-all:
    cd "{{script_dir}}/libkmp" && ./gradlew :lau-kmp:allTests