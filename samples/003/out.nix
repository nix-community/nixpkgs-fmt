# asserts are on their own lines
{ stdenv, system }:
assert system == "i686-linux";
stdenv.mkDerivation { }
