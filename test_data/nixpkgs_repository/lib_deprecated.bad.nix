{
  innerClosePropagation = acc: xs:
    if xs == []
    then acc
    else let y  = head xs;
             ys = tail xs;
         in if ! isAttrs y
            then innerClosePropagation acc ys
            else let acc' = [y] ++ acc;
                 in innerClosePropagation
                      acc'
                      (uniqList { inputList = (maybeAttrNullable "propagatedBuildInputs" [] y)
                                           ++ (maybeAttrNullable "propagatedNativeBuildInputs" [] y)
                                           ++ ys;
                                  acc = acc';
                                }
                      );
}
