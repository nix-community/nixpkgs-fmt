{
  foo = (
(
(
92
)
)
);
  xxx = listToAttrs (concatMap (name:
  let a = 4; in 5
));
foldAttrs = op: nul: list_of_attrs:
  fold (n: a:
    fold (name: o:
   o // { ${name} = op n.${name} (a.${name} or nul); }
  ) a (attrNames n)
  ) {} list_of_attrs;
bar = fun "arg"
(callPackage ./. {
  inherit foo;
});
}
