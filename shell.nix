let
  pkgs = import ./nix/nixpkgs.nix;

  inherit (pkgs)
    stdenv
    darwin
    ;
  rust =
    (pkgs.rustChannelOf {
      date = "2019-09-13";
      channel = "nightly";
      sha256 = "06881g7ba2hzmfq5vaz888d2q762zf4bxjc621rw3g8z702ps7w9";
    }).rust.override {
      extensions = [
        "clippy-preview"
        "rls-preview"
        "rustfmt-preview"
        "rust-analysis"
        "rust-std"
        "rust-src"
      ];
      targets = [ "wasm32-unknown-unknown" ];
    };
in
pkgs.mkShell {
  buildInputs = [
    pkgs.cargo-fuzz
    pkgs.gitAndTools.git-extras
    pkgs.gitAndTools.pre-commit
    pkgs.mdsh
    pkgs.openssl
    pkgs.pkgconfig
    pkgs.wasm-pack
    rust
  ]
  ++ stdenv.lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ]
  ;

  shellHook = ''
    export PATH=$PWD/target/debug:$PATH
  '';
}
