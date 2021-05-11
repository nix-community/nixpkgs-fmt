{
/* 
  decorated
  block
  comment
*/
foo = 92;
  /* the closing block comment
     should not in the newline */
  toPretty = {
      /* the closing block comment
       * should be aligned with the opening block comment
       */
      allowPrettyValues ? false
    }@args:
    let a = 0; in a;
  /* Construct a binary search path (such as $PATH) containing the
     binaries for a set of packages.
     Example:
       makeBinPath ["/root" "/usr" "/usr/local"]
       => "/root/bin:/usr/bin:/usr/local/bin"
  */
makeBinPath = makeSearchPathOutput "bin" "bin";
/* concatMapAttrsToList :: (string -> any -> [any]) -> set -> [any]
 *
 * Like mapAttrsToList, but allows multiple values to be returned
 * from the mapping function as a list.
 */
concatMapAttrsToList = mapper: attrs: concatLists (mapAttrsToList mapper attrs);
}