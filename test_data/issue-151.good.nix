let
  aaa =
    { foo, bar }:
    foo + bar
  ;
  bbb = { foo, bar }:
    foo + bar
  ;
  ccc =
    { foo, bar }:
    let x = foo + bar; in x;
  ddd =
    { foo, bar }:
    {
      inherit foo bar;
    };
in
null
