{
  fixedWidthString = width: filler: str:
    let
      strw = lib.stringLength str;
      reqWidth = width - (lib.stringLength filler);
    in
    assert lib.assertMsg
      (strw <= width)
      "fixedWidthString: requested string length (${
          toString width}) must not be shorter than actual length (${
          toString strw})";
    if strw == width then str else filler + fixedWidthString reqWidth filler str;
}
