{ lib }:
# Operations on attribute sets.
rec {
  filterAttrs = pred: set:
    listToAttrs (concatMap (name: let v = set.${name}; in if pred name v then [ (nameValuePair name v) ] else []) (attrNames set));

  inherit (self.trivial) id const pipe concat or and bitAnd bitOr bitXor;
}
