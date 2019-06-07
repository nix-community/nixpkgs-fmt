#!/usr/bin/env bash

cd "$(dirname "$0")"

wasm-pack build --target web
