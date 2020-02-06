{ foo, bar }:
let
  foo = let x = y; in
    z;
  #all in one line
  bar = let x = 3; in
    x;
in
body
