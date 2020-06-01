{
  packagesWithUpdateScriptAndMaintainer = maintainer':
    let
      maintainer =
        if ! builtins.hasAttr maintainer' pkgs.lib.maintainers then
          builtins.throw "Maintainer with name `${maintainer'} does not exist in `maintainers/maintainer-list.nix`."
        else
          builtins.getAttr maintainer' pkgs.lib.maintainers;
    in
      packagesWith (name: pkg: builtins.hasAttr "updateScript" pkg &&
                                 (if builtins.hasAttr "maintainers" pkg.meta
                                   then (if builtins.isList pkg.meta.maintainers
                                           then builtins.elem maintainer pkg.meta.maintainers
                                           else maintainer == pkg.meta.maintainers)
                                   else false)
                   )
                   (name: pkg: pkg)
                   pkgs;

  packagesWithUpdateScript = path:
    let
      attrSet = pkgs.lib.attrByPath (pkgs.lib.splitString "." path) null pkgs;
    in
      if attrSet == null then
        builtins.throw "Attribute path `${path}` does not exists."
      else
        packagesWith (name: pkg: builtins.hasAttr "updateScript" pkg)
                       (name: pkg: pkg)
                       attrSet;
}
