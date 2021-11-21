{ system ? builtins.currentSystem
, inputs ? import ./flake.lock.nix { }
}:
let
  nixpkgs = import inputs.nixpkgs {
    inherit system;
    config = { };
    overlays = [
      (final: prev: {
        fenix = import inputs.fenix {
          pkgs = prev;
          rust-analyzer-src = throw "not used";
        };
      })
    ];
  };

  rustToolchain = with nixpkgs.fenix;
    combine [
      minimal.rustc
      minimal.cargo
      targets."wasm32-unknown-unknown".latest.rust-std
    ];

  # This is a magic shell that can be both built and loaded as a nix-shell.
  mkShell = { name ? "shell", packages ? [ ], shellHook ? "" }:
    let
      drv = nixpkgs.buildEnv {
        inherit name;
        # TODO: also add the shellHook as an activation script?
        paths = packages;
      };
    in
    drv.overrideAttrs (old: {
      nativeBuildInputs = old.nativeBuildInputs ++ packages;
      inherit shellHook;
    });

  cargoToml = with builtins; (fromTOML (readFile ./Cargo.toml));
in
rec {
  inherit nixpkgs;

  nixpkgs-fmt = nixpkgs.pkgs.rustPlatform.buildRustPackage {
    inherit (cargoToml.package) name version;

    src = nixpkgs.lib.cleanSource ./.;

    doCheck = true;

    cargoLock.lockFile = ./Cargo.lock;
  };

  # This used to be the output when we were using flake-compat.
  defaultNix = nixpkgs-fmt;

  devShell = mkShell {
    packages = [
      nixpkgs.cargo-fuzz
      nixpkgs.gitAndTools.git-extras
      nixpkgs.gitAndTools.pre-commit
      nixpkgs.mdsh
      nixpkgs.openssl
      nixpkgs.pkgconfig
      nixpkgs.stdenv.cc
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
