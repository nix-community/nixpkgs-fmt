{
  fooA = fun.f a b (with c; d);
  fooB = fun (with a; b) c d;
  fooC = fun a (with b; c) d;
  fooD = fun a b (with c; {
    inherit d;
  });
  fooF = fun (with a; b)
    c d;
  fooG = fun (with a; b) c
    d;
  fooH = fun a (with b; c)
    d;
  fooI = fun a
    (with b; c) d;
  fooJ = fun a
    (with b; c) d.a;
  fooK = fun a;
  fooL = fun
    a;


  barA = fun a b {inherit d;};
  barB = fun {inherit b;} c d;
  barC = fun a {inherit c;} d;
  barD = fun a b {
    inherit d;
  };
  barF = fun {inherit b;} c
    d;
  barG = fun {inherit b;} c
    d;
  barH = fun a {inherit c;}
    d;
  barI = fun a
    {inherit c;} d;
}
