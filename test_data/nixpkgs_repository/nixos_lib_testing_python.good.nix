{
  driver = let warn = if skipLint then lib.warn "Linting is disabled!" else lib.id; in
    warn (
      runCommand testDriverName
        {
          buildInputs = [ makeWrapper ];
          testScript = testScript';
          preferLocalBuild = true;
          testName = name;
        }
        ''
          mkdir -p $out/bin
          echo -n "$testScript" > $out/test-script
          ${lib.optionalString (!skipLint) ''
            ${python3Packages.black}/bin/black --check --diff $out/test-script
          ''}
          ln -s ${testDriver}/bin/nixos-test-driver $out/bin/
          vms=($(for i in ${toString vms}; do echo $i/bin/run-*-vm; done))
          wrapProgram $out/bin/nixos-test-driver \
            --add-flags "''${vms[*]}" \
            ${lib.optionalString enableOCR
            "--prefix PATH : '${ocrProg}/bin:${imagemagick_tiff}/bin'"} \
            --run "export testScript=\"\$(${coreutils}/bin/cat $out/test-script)\"" \
            --set VLANS '${toString vlans}'
          ln -s ${testDriver}/bin/nixos-test-driver $out/bin/nixos-run-vms
          wrapProgram $out/bin/nixos-run-vms \
            --add-flags "''${vms[*]}" \
            ${lib.optionalString enableOCR "--prefix PATH : '${ocrProg}/bin'"} \
            --set tests 'start_all(); join_all();' \
            --set VLANS '${toString vlans}' \
            ${lib.optionalString (builtins.length vms == 1) "--set USE_SERIAL 1"}
        ''
    ); # "
}
