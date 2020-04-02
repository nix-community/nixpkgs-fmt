{
  testA = [
    (fetchurl {
      url = "bla";
    })
  ];

  testB = [ (fetchurl { url = "bla"; }) ];

  testC = {
    testA1 = [
      (fetchurl {
        url = "bla";
      })
    ];

    testB1 = [ (fetchurl { url = "bla"; }) ];
  };

  testD = [
    (fetchurl {
      url = "bla";
    })
    (fetchurl {
      url = "foo";
    })
  ];

  testE = [ (fetchurl { url = "bla"; }) (fetchurl { url = "foo"; }) ];
  testF = [
    (fetchurl {
      url = "bla";
    })
    (fetchurl { url = "foo"; })
  ];
  testG = [ (fetchurl { url = "bla"; }) bar ];
  testH = [
    (fetchurl { url = "bla"; })
    bar
  ];

  testI = { stdenv, ... } @ args:
    let
      foo = "0.95";
    in
      buildLinux ({
        version = "${foo}-mptcp_v1.0.0";
        inherit bar;
      } // args);

  testJ = { stdenv, ... } @ args:
    let
      foo = "0.95";
    in
      buildLinux ({
        version = "${foo}-mptcp_v1.0.0";
        inherit bar;
      });
}
