{
  description = "A flake for building Hello World";

  edition = 201909;

  inputs.naersk = { uri = "github:nmattia/naersk"; flake = false; };

  outputs = { self, nixpkgs, naersk }:
    let
      supportedSystems = [ "x86_64-linux" "x86_64-darwin" "i686-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);
      naerskOverlay = final: prev: {
        naersk = final.callPackage naersk { };
      };
    in
    {
      overlay = final: prev: {
        nixpkgs-fmt = final.callPackage ./. { };
      };

      packages =
        forAllSystems (
          system:
          {
            nixpkgs-fmt = (import nixpkgs {
              inherit system;
              overlays = [
                naerskOverlay
                self.overlay
              ];
            }).nixpkgs-fmt;
          }
        );


      defaultPackage =
        forAllSystems (
          system:
          self.packages.${system}.nixpkgs-fmt
        );

      apps =
        forAllSystems (
          system:
          {
            nixpkgs-fmt = {
              type = "app";
              program = "${self.defaultPackage.${system}}/bin/nixpkgs-fmt";
            };
          }
        );

      defaultApp =
        forAllSystems (
          system:
          self.apps.${system}.nixpkgs-fmt
        );

    };
}
