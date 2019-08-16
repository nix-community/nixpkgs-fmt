{ stdenv, fetchFromGitHub }:
let
  pname = "hello";
  version = "1.2.3";
in
stdenv.mkDerivation {
  inherit pname version;
  src = fetchFromGitHub {
    owner = "xxx";
    repo = pname;
    rev = version;
    sha256 = "...";
  };
}
