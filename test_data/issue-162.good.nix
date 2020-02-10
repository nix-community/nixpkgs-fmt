{
  inherit (import ./pep425.nix {
    inherit lib python;
    inherit (pkgs) stdenv;
  }) selectWheel;

  foo = 3;

  inherit bar;
}
