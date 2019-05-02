# Some basic formatting
{
  empty_list = [ ];

  inline_list = [ 1 2 3 4 ];

  multiline_list = [
    1
    2
    3
    4
  ];

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
  url = "https://foobar.com";

  atoms = [ true false null ];
}
