let
  sources = import ./nix/sources.nix;

  pkgs = import sources.nixpkgs {
    config = {};
    overlays = [
      (import sources.nixpkgs-mozilla)
    ];
  };

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
    rust
    pkgs.cargo-fuzz
    pkgs.mdsh
    pkgs.openssl
    pkgs.pkgconfig
    pkgs.rustfmt
    pkgs.wasm-pack
  ] ++ stdenv.lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];

  shellHook = ''
    export PATH=$PWD/target/debug:$PATH
  '';
}

