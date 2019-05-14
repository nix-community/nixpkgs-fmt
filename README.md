# nix-fmt
Experimental universal nix formatter for nixpkgs

## Samples

the samples folder contains multiple samples, `samples/*/out.nix` is what the
tool would produce after formatting the code. All the other files in the given
folder would be inputs that produce that same output.

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

## Discussions

* [nixpkgs style guide](https://nixos.org/nixpkgs/manual/#sec-syntax)
* [On Nix expression formatting](https://discourse.nixos.org/t/on-nix-expression-formatting/1521/14)
* [[Job] Implement a `nix-fmt` formatter](https://discourse.nixos.org/t/job-implement-a-nix-fmt-formatter/2819/12)
