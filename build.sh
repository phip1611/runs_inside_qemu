#!/usr/bin/env bash

# build for x86 platform should work
set -e
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu
cargo test --target x86_64-unknown-linux-gnu

rustup component add rustfmt
rustup component add clippy
cargo fmt -- --check
cargo clippy

# build for non x86 platform should fail
set +e
rustup target add thumbv6m-none-eabi 1>/dev/null 2>&1
cargo build --target thumbv6m-none-eabi 1>/dev/null 2>&1

RETURN_CODE=$?

set -e

if [ $RETURN_CODE -ne 0 ]
then
  echo "failed as expected (wrong platform)"
else
  echo "should have failed (wrong platform)"
  exit 1
fi
