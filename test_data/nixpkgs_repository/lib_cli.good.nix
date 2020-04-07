{
  foo =
    { baz
    }:
    options:
    let
      x = baz;
      y = 1;
    in
      x - y;
}
