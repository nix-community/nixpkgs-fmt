{
  buildVM =
    nodes: configurations:

    import ./eval-config.nix {
      inherit system;
    };
}
