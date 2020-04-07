{
  overrideDerivation = drv: f:
    let
      newDrv = derivation (drv.drvAttrs // (f drv));
    in lib.flip (extendDerivation true) newDrv (
      {
        meta = drv.meta or { };
        passthru = if drv ? passthru then drv.passthru else { };
      }
      //
      (drv.passthru or { })
      //
      (
        if (drv ? crossDrv && drv ? nativeDrv)
        then {
          crossDrv = overrideDerivation drv.crossDrv f;
          nativeDrv = overrideDerivation drv.nativeDrv f;
        }
        else { }
      )
    );
}
