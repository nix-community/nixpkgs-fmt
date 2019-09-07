#!/usr/bin/env bash
#
# Run this script to prepare the release
#
set -euo pipefail

echo Build cargo

cargo build

echo Update the README

mdsh

echo Testing that everything works

nix-build

echo SUCCESS
