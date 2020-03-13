(
  builtins.foldl'
    (acc: v:
      let
        isOperator = builtins.typeOf v == "list";
        operator = if isOperator then (builtins.elemAt v 0) else acc.operator;
      in
        if isOperator then (acc // { inherit operator; }) else {
          inherit operator;
          state = operators."${operator}" acc.state (satisfiesSemver pythonVersion v);
        }
    )
    {
      operator = ",";
      state = true;
    }
    tokens
).state
