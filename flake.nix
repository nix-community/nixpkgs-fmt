{
  description = "nixpkgs-fmt";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.mozilla = { url = "github:mozilla/nixpkgs-mozilla"; flake = false; };
  inputs.naersk.url = "github:nmattia/naersk";

  outputs = { self, mozilla, nixpkgs, naersk, flake-utils }:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;

      name = "nixpkgs-fmt";

      systems = flake-utils.lib.defaultSystems;

      preOverlays = [
        (import "${mozilla}/lib-overlay.nix")
        (import "${mozilla}/rust-overlay.nix")
        naersk.overlay
      ];

      overlay = final: prev: {
        nixpkgs-fmt = rec {
          nixpkgs-fmt = final.naersk.buildPackage {
            src = self;
            root = self;
            cratePaths = [ "." ];
          };
          defaultPackage = nixpkgs-fmt;
        };
      };

      shell = { pkgs }:
        let
          inherit (pkgs) stdenv darwin;

          rustChan = pkgs.rustChannelOf {
            date = "2020-12-29";
            channel = "nightly";
            sha256 = "sha256-HEBBUpbIgjbluKyT1oKU/KvQFOBFPML9vuAHuXuhoYA=";
          };

          rust = rustChan.rust.override {
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
        };
    };
}
