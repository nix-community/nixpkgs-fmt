{foo,bar}:
let
foo = let x = y; in
z;
# all in one line
bar = let x = 3; in x;
faz = let x = 3;
y = 4; in x;
baz = let x = 3;
in x;
qaz = let x = 3; in
x;
paz = v: let x = 3; in
x;
qux = v: let y = 3; in y;
nux = v: let x = 3;
y = 3; in y;
in body
