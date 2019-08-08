# nixpkgs-fmt

[![Build Status](https://travis-ci.com/nix-community/nixpkgs-fmt.svg?branch=master)](https://travis-ci.com/nix-community/nixpkgs-fmt) [![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

**STATUS: alpha**

This project's goal is to provide a nix code formatter that would be applied
on nixpkgs. Ideally automatically with a tool like ofborg.

## Demo

Try nixpkgs-fmt in your browser and submit code samples:
https://nix-community.github.io/nixpkgs-fmt/

## Design decisions

Use Rust because ofborg is written in rust. Rust also has a small chance of
being included in nix upstream.

Use an incremental and rule-based formatter (vs pretty-printing). This allows
to format partial code and leave more expressivity to the developer. For
example double new-lines can be used when the developer wants a section of
code to stand out. The important part is to avoid discussions on indent and
brackets alignment.

Favour mergeability. nixpkgs is seeing a lot of traffic. Spread things out
vertically to minimize the chances of merge conflicts.

## Usage

`$ nixpkgs-fmt --help 2>&1`
```
nixpkgs-fmt 0.1
Format Nix code

USAGE:
    nixpkgs-fmt [FLAGS] [OPTIONS] [FILE]

FLAGS:
    -h, --help        Prints help information
    -i, --in-place    Overwrite FILE in place
        --parse       Show syntax tree instead of reformatting
    -V, --version     Prints version information

OPTIONS:
    -o, --output <file>    Place the output into <file>

ARGS:
    <FILE>    File to reformat

```

## Development

Install Rust and Cargo or run `nix-shell` to load the project dependencies.

Install [pre-commit](https://pre-commit.com/) and run `pre-commit install` to
setup the git hooks on the repository. This will allow to keep the code nicely
formatted over time.

Then use `cargo run` to build and run the software.

### Running Fuzzer

```
$ cargo install cargo-fuzz
$ mkdir -p ./fuzz/corpus/fmt
$ cp test_data/**.nix ./fuzz/corpus/fmt
$ rustup run nightly -- cargo fuzz run fmt
```

or with nix:

```
$ nix-shell --run "cargo fuzz run fmt"
```

* `fmt` is the name of the target in `./fuzz/Cargo.toml`

Fuzzer will run indefinitelly or until it finds a crash.
The crashing input is written to `fuzz/artifacts` directory.
Commit this `crash-` file, and it will be automatically tested by a unit-test.

## Documentation

* [HOWTO write new rules](docs/howto_rules.md)
* [HOWTO WASM](wasm/README.md)

## Related projects

* [hnix](https://github.com/haskell-nix/hnix) - Haskell implementation of Nix
  including a parser. The parser is not comment-preserving.
* [rnix](https://gitlab.com/jD91mZM2/rnix) - Rust Nix parser based on
  [rowan](https://github.com/rust-analyzer/rowan)
* [nix-lsp](https://gitlab.com/jD91mZM2/nix-lsp) - Nix language server using
  rnix
* [tree-sitter-nix](https://github.com/cstrahan/tree-sitter-nix) - Tree Sitter
  is a forgiving parser used by Atom for on-the-fly syntax highlighting and
  others. This is a implementation for Nix.
* [format-nix](https://github.com/justinwoo/format-nix/) - A nix formatter
  using tree-sitter-nix.
* [nixfmt](https://github.com/serokell/nixfmt) - A nix formatter written in
  Haskell.
* [nix-fmt](https://github.com/jmackie/nix-fmt)
* [nix-format](https://github.com/taktoa/nix-format) - Emacs-based Nix formatter
* [nix-beautify](https://github.com/nixcloud/nix-beautify)
* [nix-linter](https://github.com/Synthetica9/nix-linter)
* [canonix](https://github.com/hercules-ci/canonix/) - Nix formatter prototype written in Haskell using tree-sitter-nix grammar.

## Discussions

* [nixpkgs style guide](https://nixos.org/nixpkgs/manual/#sec-syntax)
* [On Nix expression formatting](https://discourse.nixos.org/t/on-nix-expression-formatting/1521/14)
* [[Job] Implement a `nix-fmt` formatter](https://discourse.nixos.org/t/job-implement-a-nix-fmt-formatter/2819/12)
