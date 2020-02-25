{ python3 }:
python3.pkgs.buildPythonApplication {
 propagatedBuildInputs = (with python3.pkgs; [
 pkg1
]);
}
