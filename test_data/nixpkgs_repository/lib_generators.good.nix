{
  toINI =
    {
      # apply transformations (e.g. escapes) to section names
      mkSectionName ? (name: libStr.escape [ "[" "]" ] name)
    }: attrsOfAttrs:
    let
      # map function to string for each key val
      mapAttrsToStringsSep = sep: mapFn: attrs:
        libStr.concatStringsSep sep
        (libAttr.mapAttrsToList mapFn attrs);
      mkSection = sectName: sectValues: ''
        [${mkSectionName sectName}]
      '' + toKeyValue { inherit mkKeyValue listsAsDuplicateKeys; } sectValues;
    in
      # map input to ini sections
      mapAttrsToStringsSep "\n" mkSection attrsOfAttrs;

  expr = ind: x: with builtins;
    if x == null then "" else
    if isBool x then bool ind x else
    if isInt x then int ind x else
    if isString x then str ind x else
    if isList x then list ind x else
    if isAttrs x then attrs ind x else
    if isFloat x then float ind x else
    abort "generators.toPlist: should never happen (v = ${v})";
}
