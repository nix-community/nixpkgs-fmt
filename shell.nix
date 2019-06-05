with import <nixpkgs> {};
mkShell {
  buildInputs = [
    cargo
  ] ++ stdenv.lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
}

