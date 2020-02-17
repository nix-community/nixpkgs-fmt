{
foo = if bar != null
then bar
      else baz;
bar =
if bar == null
then foo
else baz;
qux = if bux == null then nux else baz;
nux = if foo == baz then bar
else bar;
}
