{
  boot.initrd.preLVMCommands = mkBefore ''
      kbd_mode ${if isUnicode then "-u" else "-a"} -C /dev/console
      printf "\033%%${if isUnicode then "G" else "@"}" >> /dev/console
      loadkmap < ${optimizedKeymap}
      ${optionalString cfg.earlySetup ''
    setfont -C /dev/console $extraUtils/share/consolefonts/font.psf
    ''}
  '';
}
