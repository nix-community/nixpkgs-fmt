let
  sources = import ./sources.nix;

  pkgs = import sources.nixpkgs {
    config = {};
    overlays = [
      (import sources.nixpkgs-mozilla)
    ];
  };
in
pkgs
