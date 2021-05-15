{
  description = "nixpkgs-fmt";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.mozilla = { url = "github:mozilla/nixpkgs-mozilla"; flake = false; };
  inputs.naersk.url = "github:nmattia/naersk";

  outputs = { self, mozilla, nixpkgs, naersk, flake-utils }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import ./. { inherit system inputs; }; in
      {
        defaultPackage = pkgs.nixpkgs-fmt;
        legacyPackages = pkgs;
        devShell = pkgs.devShell;
      }
    );
}
