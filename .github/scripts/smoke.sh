#!/usr/bin/env bash
# Compile examples/query.c against the release library and run it.
set -euo pipefail

target="${1:?target triple required}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

dir="target/${target}/release"
case "$(uname -s)" in
  Linux)
    cc -I include examples/query.c "$dir/libhwinfo.so" -Wl,-rpath,'$ORIGIN' -o "$dir/hwinfo_query"
    "$dir/hwinfo_query"
    ;;
  Darwin)
    clang -I include examples/query.c "$dir/libhwinfo.dylib" -Wl,-rpath,@executable_path -o "$dir/hwinfo_query"
    "$dir/hwinfo_query"
    ;;
  MINGW* | MSYS* | CYGWIN*)
    cl -nologo -I include examples/query.c -Fe:"$dir/hwinfo_query.exe" -link "$dir/hwinfo.dll.lib"
    "$dir/hwinfo_query.exe"
    ;;
  *)
    echo "unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac
