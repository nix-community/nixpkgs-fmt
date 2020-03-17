with import <nixpkgs> { };
stdenv.mkDerivation {
  name = "rnix";
  buildInputs = [ cargo ];
}
