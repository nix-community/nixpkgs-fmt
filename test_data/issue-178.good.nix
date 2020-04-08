{
  testA = (buildCargoCrate {
    name = "nixpkgs-fmt";
    # TODO: probably want to filter .gitignore or something
    src = sources.nixpkgs-fmt;
  }).nixpkgs-fmt.build;

  testB = buildLinux (map ({ name }:
    { inherit name; }
  ) cfg.feeds);
}
