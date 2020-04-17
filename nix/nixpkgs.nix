let
  sources = import ./sources.nix;
  pkgs = import sources.nixpkgs {
    config = { };
    overlays = [
      (import sources.nixpkgs-mozilla)
      (self: pkgs: {
        naersk = pkgs.callPackage sources.naersk { };
      })
    ];
  };
in
pkgs
