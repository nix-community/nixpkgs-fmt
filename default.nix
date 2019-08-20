{ pkgs ? import ./nix/nixpkgs.nix }:
let
  lib = pkgs.lib;

  assertMsg = pred: msg:
    if pred then true
    else builtins.trace "assert failed: ${msg}" false;

  assertEqual = left: right:
    assertMsg (left == right) "expected ${toString left} to equal ${toString right}";

  # another attempt to make filterSource nicer to use
  allowSource = { allow, src }:
    let
      out = builtins.filterSource filter src;
      filter = path: _fileType:
        lib.any (checkElem path) allow;
      checkElem = path: elem:
        lib.hasPrefix (toString elem) (toString path);
    in
      out;

  # get some metadata from ./Cargo.toml
  meta = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  # calculate the hash of ./Cargo.lock
  cargoLockHash = builtins.hashString "sha256" (builtins.readFile ./Cargo.lock);
in
pkgs.rustPlatform.buildRustPackage {
  pname = meta.package.name;
  version = meta.package.version;

  src = allowSource {
    allow = [
      ./Cargo.lock
      ./Cargo.toml
      ./fuzz
      ./src
      ./test_data
      ./wasm
    ];
    src = ./.;
  };

  # update both values whenever Cargo.lock changes
  cargoSha256 =
    assert (assertEqual cargoLockHash "7f3ca77b9001dca3aa8972099a51afd27a33d32629793b75caa1f5d4f3bf2f67");
    "0p3qa1asdvw2npav4281lzndjczrzac6fr8z4y61m7rbn363s8sa";
}
