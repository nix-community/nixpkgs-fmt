{ system ? builtins.currentSystem
, inputs ? import ./flake.lock.nix { }
}:
let
  nixpkgs = import inputs.nixpkgs {
    inherit system;
    config = { };
    overlays = [
      (import "${inputs.mozilla}/lib-overlay.nix")
      (import "${inputs.mozilla}/rust-overlay.nix")
      (import "${inputs.naersk}/overlay.nix")
    ];
  };

  rustChan = nixpkgs.rustChannelOf {
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
rec {
  nixpkgs-fmt = nixpkgs.naersk.buildPackage {
    src = ./.;
    root = ./.;
    cratePaths = [ "." ];
  };

  defaultPackage = nixpkgs-fmt;

  devShell = nixpkgs.mkShell {
    nativeBuildInputs = [
      nixpkgs.cargo-fuzz
      nixpkgs.gitAndTools.git-extras
      nixpkgs.gitAndTools.pre-commit
      nixpkgs.mdsh
      nixpkgs.openssl
      nixpkgs.pkgconfig
      nixpkgs.wasm-pack
      rust
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
