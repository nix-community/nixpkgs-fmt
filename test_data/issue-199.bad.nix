let
# comment


a = (if true then 1 else 2);
b = "${if true then "1" else "2"}";


# comment


c = ''
  ${foo (if true then ''
    bla
  '' else ''
    blub
  '')}
'';

d = ''
  ${if true then ''
    bla
  '' else ''
    blub
  ''}
'';




in
{}
