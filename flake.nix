{
  description = "nixpkgs-fmt flake";

  inputs.naersk = { url = "github:nmattia/naersk"; };

  outputs = { self, nixpkgs, naersk }:
    let
      supportedSystems = [ "x86_64-linux" "x86_64-darwin" "i686-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems f;
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
                naersk.overlay
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
