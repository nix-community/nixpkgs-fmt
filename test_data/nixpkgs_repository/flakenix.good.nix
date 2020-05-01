{
  lib = lib // {
    nixosSystem = { modules, ... } @ args:
      import ./nixos/lib/eval-config.nix (args // {
        modules = modules ++
          [
            {
              system.nixos.versionSuffix =
                ".${lib.substring 0 8 self.lastModified}.${self.shortRev or "dirty"}";
              system.nixos.revision = lib.mkIf (self ? rev) self.rev;
            }
          ];
      });
  };
}
