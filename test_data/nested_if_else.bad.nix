{
  modifyA = n: fn: v: if (n == 0) then fn v
                      else if isList  v then map (modify (n - 1) fn) v
                      else if isAttrs v then mapAttrs
   (const (modify (n - 1) fn)) v
       else v;
  modifyB = n: fn: v:
  if (n == 0) then fn v
  else if isList  v then map (modify (n - 1) fn) v
  else if isAttrs v then mapAttrs
   (const (modify (n - 1) fn)) v
  else v;
}
