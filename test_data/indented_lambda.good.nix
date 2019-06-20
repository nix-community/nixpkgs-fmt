{
  toINI = {
    # parameter comment
    mkSectionName ? (name: libStr.escape [ "[" "]" ] name)
  , mkKeyValue ? mkKeyValueDefault {} "="
  }: attrsOfAttrs:
    mapAttrsToStringsSep "\n" mkSection attrsOfAttrs;
}
