let
  pkgs = import ./nix/nixpkgs.nix;

  inherit (pkgs)
    stdenv
    darwin
    ;

  rust = pkgs.latest.rustChannels.nightly.rust.override {
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
    pkgs.rustfmt
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
