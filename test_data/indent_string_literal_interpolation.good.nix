{
  postFixup = ''
    cd $out/clion-${version}
    rm -rf bin/cmake/linux
    ln -s ${cmake} bin/cmake/linux

    lldbLibPath=$out/clion-${version}/bin/lldb/linux/lib
    interp="$(cat $NIX_CC/nix-support/dynamic-linker)"
  '';
}
