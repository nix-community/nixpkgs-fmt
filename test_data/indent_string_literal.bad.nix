{
  python =
''
for i in range(10):
    print(i)
'';

  python_indented = ''
  for i in range(10):
      print(i)
'';

  python_last_line1 =
''
for i in range(10):
    print(i)'';

  python_last_line2 =
''
for i in range(10):
    print(i)
 '';

  unindetable =
''python
for i in range(10):
    print(i)
'';

nix.extraOptions = ''
  builders-use-substitutes = true
'';

  empty = '''';
  blank = '' '';
}
