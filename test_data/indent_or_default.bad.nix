{
catAttrs = builtins.catAttrs or
(attr: l: concatLists (map (s: if s ? ${attr} then [ s.${attr} ] else []) l));
}
