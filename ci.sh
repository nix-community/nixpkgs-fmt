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

# build nixpkgs-fmt
run cargo build --verbose

# run after build, pre-commit needs nixpkgs-fmt
run pre-commit run --all-files

# run the tests
run cargo test --verbose

# generate the webassembly page
run ./wasm/build.sh

