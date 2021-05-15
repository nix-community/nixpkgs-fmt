{ system ? builtins.currentSystem
, inputs ? import ./flake.lock.nix { }
}:
let
  nixpkgs = import inputs.nixpkgs {
    inherit system;
    config = { };
    overlays = [
      (import "${inputs.naersk}/overlay.nix")
      (import inputs.fenix)
    ];
  };

  rustToolchain = with nixpkgs.fenix;
    combine [
      minimal.rustc
      minimal.cargo
      targets."wasm32-unknown-unknown".latest.rust-std
    ];

  naersk = nixpkgs.naersk.override {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };

in
rec {
  inherit nixpkgs;

  nixpkgs-fmt = nixpkgs.naersk.buildPackage {
    src = ./.;
    root = ./.;
    cratePaths = [ "." ];
  };

  devShell = nixpkgs.mkShell {
    nativeBuildInputs = [
      nixpkgs.cargo-fuzz
      nixpkgs.gitAndTools.git-extras
      nixpkgs.gitAndTools.pre-commit
      nixpkgs.mdsh
      nixpkgs.openssl
      nixpkgs.pkgconfig
      nixpkgs.wasm-pack
      rustToolchain
    ]
    ++ nixpkgs.lib.optionals nixpkgs.stdenv.isDarwin [
      nixpkgs.darwin.apple_sdk.frameworks.Security
    ]
    ;

    shellHook = ''
      export PATH=$PWD/target/debug:$PATH
    '';
  };
}
