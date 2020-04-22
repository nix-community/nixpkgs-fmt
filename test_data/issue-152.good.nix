let
  id =
    x:
    # comment
    x;
  foo = x:
    #comment
    x + x;
  bar = x:
    let y = x;
    in
    #comment
    x;
  baz = x:
    #comment
    y:
    #comment
    foo + bar;
in
#comment
id 1
