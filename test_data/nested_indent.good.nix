{
  pkgs ? import ./nix/nixpkgs.nix {}
, src ? builtins.fetchGit {
    url = ./.;
    ref = "HEAD";
  }
}:
pkgs.example rec {}
