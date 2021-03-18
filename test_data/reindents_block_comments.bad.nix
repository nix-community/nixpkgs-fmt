{
/* 
decoreted
block
comment
*/
foo = 92;
    /* the closing block comment
       should not in the newline */
  toPretty = {
    /* the closing block comment
       should be aligned with the opening block comment
       */
      allowPrettyValues ? false
    }@args:
    let a = 0; in a;
}
