{
  matchAttrs = pattern: attrs: assert isAttrs pattern;
fold and true (attrValues (zipAttrsWithNames (attrNames pattern) (n: values:
      let pat = head values; val = head (tail values); in
if length values == 1 then false
        else if isAttrs pat then isAttrs val && matchAttrs pat val
        else pat == val
) [ pattern attrs ]));
}
