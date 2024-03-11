{ lib, hello }:
let
  hello-hardened = hello.overrideAttrs (oldAttrs: {
    pname = oldAttrs.pname ++ "-hardened";
  });
in
builtins.trace "Hardened hello‮⁦-harden⁩‬" hello
