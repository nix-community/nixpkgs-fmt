# nixpkgs-fmt-wasm

This package is re-exporting the nixpkgs-fmt library to WASM. The goal is to
provide a demo page for users to test and report issues.

## Dependencies

Run `nix-shell` in the parent folder if you are fortunate enough to have Nix
installed. Otherwise go to https://rustwasm.github.io/wasm-pack/installer/ and
follow the installation instructions there.

## Building

Run `./build.sh` to build the WASM target. It's going to take a while.

Once the project has finished to build, all the outputs will be under `./pkg`.

## Running

Use a static file server like [caddy](https://caddyserver.com/) to serve the
demo. Start `caddy` and then go to http://localhost:2015/ . Enjoy!

## TODO

* Automatically publish master to github pages
* Submit formatting as GitHub issues
