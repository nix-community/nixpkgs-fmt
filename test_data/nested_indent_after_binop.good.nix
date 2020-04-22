pkgs.example rec {
  buildInputs = [ pkgs.foo ] ++
    pkgs.stdenv.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.bar
    ];
}
