let
  a = {
    inherit foo;inherit bar;
  };
  b = {
    inherit foo;# Comment
  };
  c = {
    inherit foo;bar = baz;
  };
in
{}
