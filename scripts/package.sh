#!/bin/sh
# scripts/package.sh [OUT]: builds Overeaten for aarch64 Gizlet OS and
# assembles the app folder that `gizlet install` takes (dist/overeaten by
# default).
#
# Needs zig, the Rust target aarch64-unknown-linux-gnu and Gizlet runtime 1's
# libraylib.so: set GIZLET_RAYLIB_LIB to the folder holding it. gizlet/os
# builds it with `bazel build --config=r36s //runtime:raylib`.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-$root/dist/overeaten}
target=aarch64-unknown-linux-gnu

: "${GIZLET_RAYLIB_LIB:?set GIZLET_RAYLIB_LIB to the folder holding runtime 1 libraylib.so}"
[ -f "$GIZLET_RAYLIB_LIB/libraylib.so" ] || { echo "no libraylib.so in $GIZLET_RAYLIB_LIB" >&2; exit 1; }

# bindgen reads raylib.h for the device: give it zig's glibc headers.
zig_lib=$(zig env | sed -n -e 's/.*\.lib_dir = "\(.*\)".*/\1/p' -e 's/.*"lib_dir": *"\([^"]*\)".*/\1/p')
inc=$zig_lib/libc/include
[ -d "$inc/generic-glibc" ] || { echo "zig's glibc headers are missing under $inc" >&2; exit 1; }
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$root/scripts/zig-cc"
export CC_aarch64_unknown_linux_gnu="$root/scripts/zig-cc"
export BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_gnu="-isystem $inc/aarch64-linux-gnu -isystem $inc/generic-glibc -isystem $inc/any-linux-any"

cd "$root"
cargo build --release --locked --target "$target" --features gizlet-os

rm -rf "$out"
mkdir -p "$out/bin" "$out/licenses"
cp "target/$target/release/overeaten" "$out/bin/overeaten"
cp gizlet.toml "$out/gizlet.toml"
cp assets/icon.png "$out/icon.png"
mkdir -p "$out/assets"
cp -R assets/art assets/audio "$out/assets/"
cp LICENSE "$out/licenses/overeaten-MIT.txt"
cp assets/fonts/noto_ofl.txt "$out/licenses/noto-sans-sc-OFL.txt"
echo "$out"
