{
  imports = [
    # Create an alias for the "enable" option.
    (mkAliasOptionModule [ "enableAlias" ] [ "enable" ])

    # Disable the aliased option with a high priority so it
    # should override the next import.
    ({ config, lib, ... }:
      {
        enableAlias = lib.mkForce false;
      }
    )

    # Enable the normal (non-aliased) option.
    ({ config, lib, ... }:
      {
        enable = true;
      }
    )
  ];
}
