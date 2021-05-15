{ system ? builtins.currentSystem
, inputs ? import ./flake.lock.nix { }
}:
let
  nixpkgs = import inputs.nixpkgs {
    inherit system;
    config = { };
    overlays = [
      (import inputs.fenix)
    ];
  };

  rustToolchain = with nixpkgs.fenix;
    combine [
      minimal.rustc
      minimal.cargo
      targets."wasm32-unknown-unknown".latest.rust-std
    ];

  naersk = nixpkgs.callPackage inputs.naersk {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };

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

in
rec {
  inherit nixpkgs;

  nixpkgs-fmt = naersk.buildPackage {
    src = ./.;
    root = ./.;
    cratePaths = [ "." ];
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
