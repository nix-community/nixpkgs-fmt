# nixpkgs-fmt - Nix code formatter for nixpkgs

[![Build Status](https://travis-ci.com/nix-community/nixpkgs-fmt.svg?branch=master)](https://travis-ci.com/nix-community/nixpkgs-fmt) [![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

**STATUS: beta**

This project's goal is to format the nix code in nixpkgs to increase the
consistency of the code found there. Ideally automatically with pre-commit
hooks and later ofborg enforcing the format.

## Demo

You can try nixpkgs-fmt in your browser. The page also provides a way for you
to submit code samples if you find the output not satisfying:
https://nix-community.github.io/nixpkgs-fmt/

## Design decisions

You migth ask yourself; why do we need yet another nix code formatter?

The main goal of nixpkgs-fmt is to provide some overall consistency in the
nix code submitted to [nixpkgs](https://github.com/NixOS/nixpkgs), our main
package repository.

At this point it's important to understand that there are multiple possible
outputs for a code formatter. Those outputs will depend on multiple
conflicting desires and depending on how much weight is being put on each
requirement the output will change.

For nixpkgs-fmt we have a few of these:

1. Minimize merge conflicts. nixpkgs is seein a lot of pull-requests and we
   want to avoid them getting unecessarily stable.
2. Only expand, don't collapse. It's up to the developer to choose if an
   element should be on a single line or multiple lines.
3. Respect the developer's expressivity. Empty lines can be useful as a way to
   separate blocks of code.
4. Only change the indent of one (+/-) per line. Not sure why but it seems
   like a good thing.

Corrolary rules:

* because of (1). The format is quite close to what exists in nixpkgs already.
* because of (1). Don't align values vertically, a single line change can
  introduce a very big diff.
* because of (1). Avoid too many rules. More rules means more formatting
  changes that create merge conflicts.
* because of (2). Don't enforce line lengths. Line length limits also create
  complicated heursistics.

At the time where we started this project none of the other formatters were
weighted that way.

To implement this, we needed a whitespace and comment-preserving parser which
[rnix][rnix] provides to us. Then create an engine that follows the AST and
patches the tree with rewrite rules. The nice thing about this design is that
it also works on incomplete or broken nix code. We are able to format up to
the part that is missing/broken, which makes it great for potential editor
integration.

Most of the other formatters out there take a pretty-printing approach where
the AST is parsed, and then a pretty-printer inspects and formats the AST back
to code without taking spaces and newlines into account. The advantage is that
it's initially easier to implement. The output is very strict and the same AST
will always give the same output. One disadvantage is that the pretty-printer
needs to handle all the possible combination of Nix code to make them look
good.

With nixpkgs-fmt the output will depend on how the code was formatted
initially. The developer still has some input on how they want to format their
code. If there is no rule for a complicated case, the code will be left alone.
For nixpkgs this approach will be preferable since it minimizes the diff.

Well done for reading all of this, I hope this clarifies a bit why nixpkgs-fmt
exists and what role it can play.

## Usage

`$ nixpkgs-fmt --help 2>&1 || true`
```
nixpkgs-fmt 0.1
Format Nix code

USAGE:
    nixpkgs-fmt [FLAGS] [FILE]...

FLAGS:
    -h, --help        Prints help information
    -i, --in-place    Overwrite FILE in place
        --parse       Show syntax tree instead of reformatting
    -V, --version     Prints version information

ARGS:
    <FILE>...    File to reformat

```
## Installation

nixpkgs-fmt is available in nixpkgs master. `nix-env -i nixpkgs-fmt`.

It's also possible to install it directly from this repository:

`nix-env -f https://github.com/nix-community/nixpkgs-fmt/archive/master.tar.gz -i`

### pre-commit hook

This project can also be installed as a [pre-commit](https://pre-commit.com/)
hook.

Add to your project's `.pre-commit-config.yaml`:

```yaml
-   repo: https://github.com/nix-community/nixpkgs-fmt
    rev: master
    hooks:
    -   id: nixpkgs-fmt
```

Make sure to have rust available in your environment.

Then run `pre-commit install-hooks`

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
* [How we do releases](docs/releasing.md)

## Related projects

Feel free to submit your project!

### Formatters

* [canonix](https://github.com/hercules-ci/canonix/) - Nix formatter prototype written in Haskell using the tree-sitter-nix grammar.
* [format-nix](https://github.com/justinwoo/format-nix/) - A nix formatter using tree-sitter-nix.
* [nix-format](https://github.com/taktoa/nix-format) - Emacs-based Nix formatter.
* [nix-lsp](https://gitlab.com/jD91mZM2/nix-lsp) - Nix language server using rnix.
* [nixfmt](https://github.com/serokell/nixfmt) - A nix formatter written in Haskell.

### Linters

* [nix-linter](https://github.com/Synthetica9/nix-linter)

### Parsers

* [hnix](https://github.com/haskell-nix/hnix) - Haskell implementation of Nix including a parser. The parser is not comment-preserving.
* [rnix](https://gitlab.com/jD91mZM2/rnix) - Rust Nix parser based on [rowan](https://github.com/rust-analyzer/rowan)
* [tree-sitter-nix](https://github.com/cstrahan/tree-sitter-nix) - Tree Sitter is a forgiving parser used by Atom for on-the-fly syntax highlighting and others. This is a implementation for Nix.

## Discussions

* [nixpkgs style guide](https://nixos.org/nixpkgs/manual/#sec-syntax)
* [On Nix expression formatting](https://discourse.nixos.org/t/on-nix-expression-formatting/1521/14)
* [[Job] Implement a `nix-fmt` formatter](https://discourse.nixos.org/t/job-implement-a-nix-fmt-formatter/2819/12)
