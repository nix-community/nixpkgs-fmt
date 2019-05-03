{ lib }:
{
  list = [
    elem1
    elem2
    elem3
  ] ++ lib.optionals stdenv.isDarwin [
    elem4
    elem5
  ];
}
