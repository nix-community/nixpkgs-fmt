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

  rust = pkgs.latest.rustChannels.stable.rust.override {
    targets = [ "wasm32-unknown-unknown" ];
  };
in
pkgs.mkShell {
  buildInputs = [
    rust
    pkgs.mdsh
    pkgs.rustfmt
    pkgs.wasm-pack
    pkgs.pkgconfig
    pkgs.openssl
  ] ++ stdenv.lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];

  shellHook = ''
    export PATH=$PWD/target/debug:$PATH
  '';
}

