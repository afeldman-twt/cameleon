#!/bin/bash
set -e

export RUST_BACKTRACE=1

cargo fmt --all -- --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo doc --no-deps

cargo test --workspace --all-targets --all-features
cargo test --workspace --all-features --doc

echo "### all tests passed! ###"
