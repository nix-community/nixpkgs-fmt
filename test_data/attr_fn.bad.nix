{
  f = { x
  , y
      }: body;

  testAllTrue = expr: {inherit expr;expected=map (x: true) expr; };
}
