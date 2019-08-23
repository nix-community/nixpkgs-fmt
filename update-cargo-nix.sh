#!/usr/bin/env bash
#
# nix-build that automatically fixes hash mismatches
#
set -euo pipefail

# hash mismatch in fixed-output derivation '/nix/store/7kahc4afal4rzf76266aflvrxf3w0nhr-nixpkgs-fmt-0.1.0-vendor':
#   wanted: sha256:0p3qa1asdvw2npav4281lzndjczrzac6fr8z5y61m7rbn363s8sa
#   got:    sha256:0p3qa1asdvw2npav4281lzndjczrzac6fr8z4y61m7rbn363s8sa
monitor_hash_mismatch() {
  local line
  while IFS="" read -r line || [ -n "$line" ]; do
    printf "err: %s\n" "$line"

    if [[ $line =~ hash\ mismatch\ in ]]; then
      read -r line
      printf "err: %s\n" "$line"
      if ! [[ $line =~ wanted:\ +sha256:([0-9a-z]+) ]]; then
        continue
      fi
      local wanted=${BASH_REMATCH[1]}

      read -r line
      printf "err: %s\n" "$line"
      if ! [[ $line =~ got:\ +sha256:([0-9a-z]+) ]]; then
        continue
      fi
      local got=${BASH_REMATCH[1]}

      local rule="s/$wanted/$got/g"
      echo "AUTO PATCHING default.nix with '$rule'"
      sed -e "$rule" -i default.nix
    fi
  done
}

echo Checking Cargo

cargo check

echo Patching the Cargo.lock hash

cargoLockHash=$(nix hash-file --base16 ./Cargo.lock)
sed -e "s/cargoLockHash = \".*\";/cargoLockHash = \"$cargoLockHash\";/" -i default.nix

echo Patching the cargoSha256

sed -e "s/cargoSha256 = \".*\";/cargoSha256 = \"1111111111111111111111111111111111111111111111111111\";/" -i default.nix

nix-build 2> >(monitor_hash_mismatch 1>&2) || true

echo Testing that everything works

nix-build

echo SUCCESS
