#!/usr/bin/env bash

# Unicode bidi (RTL) control characters
# See https://www.unicode.org/reports/tr9/tr9-42.html
UCHAR_LRE="$(printf "\\u202a")" 
UCHAR_RLE="$(printf "\\u202b")"
UCHAR_LRO="$(printf "\\u202d")"
UCHAR_RLO="$(printf "\\u202e")"
UCHAR_PDF="$(printf "\\u202c")"
UCHAR_LRI="$(printf "\\u2066")"
UCHAR_RLI="$(printf "\\u2067")"
UCHAR_FSI="$(printf "\\u2068")"
UCHAR_PDI="$(printf "\\u2069")"

cat << EOF
{ lib, hello }:
{
  hello-insecure = hello.overrideAttrs (oldAttrs: {
    meta = oldAttrs.meta // {
      /* Mark as insecure${UCHAR_RLO}${UCHAR_LRI} insecure = true;${UCHAR_PDI}${UCHAR_LRI} */
    };
  });
}
EOF
