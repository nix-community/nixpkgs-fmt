{
  buildInputs = [
  ] ++ stdenv.lib.optionals enableGui (with qt5; [ qtbase qtwebkit ])
    ++ stdenv.lib.optionals enableJupyter [ boost jsoncpp openssl zmqpp ]
  ;
}
