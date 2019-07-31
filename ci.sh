#!/usr/bin/env nix-shell
#!nix-shell -i bash
#
# This is the script executed by CI
set -euo pipefail

## Functions ##

run() {
  echo >&2
  echo "$ ${*@Q}" >&2
  "$@"
}

## Main ##

mkdir -p "${TMPDIR}"

run cargo build --verbose

run ./wasm/build.sh

run cargo test --verbose

run cargo fuzz run fmt -- -max_total_time=300
