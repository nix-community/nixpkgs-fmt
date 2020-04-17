{ python3 }:
python3.pkgs.buildPythonApplication {
  propagatedBuildInputs = (with python3.pkgs; [
    pkg1
  ]);
  testB = forAllSystems
    (
      system:
      self.apps.${system}.nixpkgs-fmt
    );
}
