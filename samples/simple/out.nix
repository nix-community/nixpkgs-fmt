# Some basic formatting
{
  empty_list = [];

  inline_list = [ 1 2 3 ];

  multiline_list = [ 1 2 3 4 ];

  inline_attrset = { x = "y"; };

  multiline_attrset = {
    a = 3;
    b = 5;
  };

  # some comment over here
  fn = x: x + x;

  relpath = ./hello;

  abspath = /hello;

  # URLs get converted to strings
  url = https://foobar.com;

  atoms = [ true false null ];

  # Combined
  listOfAttrs = [
    {
      attr1 = 3;
      attr2 = "fff";
    }
    {
      attr1 = 5;
      attr2 = "ggg";
    }
  ];

  # long expression
  attrs = {
    attr1 = short_expr;
    attr2 = if true then big_expr else big_expr;
  };
}
