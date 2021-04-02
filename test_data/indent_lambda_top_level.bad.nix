import ./make-test-python.nix ({pkgs, lib, ...}:

let
  bar = 57;
in {
  baz = qux;
})