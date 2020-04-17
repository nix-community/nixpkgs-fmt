{
  foo =
    { z
    , y ? 0
    }:
    let
      x = 1;
      y = 2;
    in
    x + y + z;

  bar =
    { a
    , b ? 0
    }:
    let
      c = 1;
      b = 2;
    in
    a + b + c;

  baz =
    { pkg
    , num ? 0
    }:
    let
      val = 1;
      num = 2;
    in
    pkg + num + val;
}
