{
  # how to string-format the option name;
  # by default one character is a short option (`-`),
  # more than one characters a long option (`--`).
  mkOptionName ?
  k: if builtins.stringLength k == 1
    then "-${k}"
    else "--${k}"
, mkOption ? k: v:
  if v == null
  then []
  else [ (mkOptionName k) (lib.generators.mkValueStringDefault {} v) ]
}:
{  toINI = {
  # parameter comment
  mkSectionName ? (name: libStr.escape [ "[" "]" ] name)
  , mkKeyValue ? mkKeyValueDefault {} "="
  }: attrsOfAttrs:
    mapAttrsToStringsSep "\n" mkSection attrsOfAttrs;
}
