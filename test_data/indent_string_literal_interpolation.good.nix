{
  postFixup = ''
    cd $out/clion-${version}
    rm -rf bin/cmake/linux
    ln -s ${cmake} bin/cmake/linux

    lldbLibPath=$out/clion-${version}/bin/lldb/linux/lib
    interp="$(cat $NIX_CC/nix-support/dynamic-linker)"
  '';

  foo = ''
    bar = ${builtins.concatStringsSep " " [
      1
      2
      3
    ]}
    bla = hoi
  '';

  bar = ''
    foo
    ${
      foo
    }
    foo
  '';

  baz =
    ''
      foo
      ${
        foo
      }
      foo
    '';

  qux =
    ''
      bar = ${builtins.concatStringsSep " " [
        1
        2
        3
      ]}
      bla = hoi
    '';

  singleAsciiDoc = value: ''
    ${
      if lib.hasAttr "example" value
      then ''
        Example::
        ${
          builtins.toJSON value.example
        }
      ''
      else "No Example:: {blank}"
    }
  '';

  nested_antiquotation =
    mkBefore
      ''
        ${optionalString cfg.earlySetup ''
          ${optionalString cfg.earlySetup ''
            setfont -C /dev/console $extraUtils/share/consolefonts/font.psf
          ''}
          setfont -C /dev/console $extraUtils/share/consolefonts/font.psf
        ''}
      '';

  singleAsciiDoc = value: ''
    Example::
    ${
      builtins.toJSON value.example
    }
  '';
}
