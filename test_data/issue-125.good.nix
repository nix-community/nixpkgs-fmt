{ foo, bar }:
let
  foo = let x = y; in
    z;
  # all in one line
  bar = let x = 3; in x;
  baz = v: let x = 3; in
    x;
  qux = v: let y = 3; in y;
  nux = v:
    let
      x = 3;
      y = 3;
    in y;
in
body
