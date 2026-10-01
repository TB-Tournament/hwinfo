#!/usr/bin/env bash
# Pack include/hwinfo.h and the release dynamic library into hwinfo-<target>.tar.gz.
set -euo pipefail

target="${1:?target triple required}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

src="target/${target}/release"
stage="dist/${target}"
rm -rf "$stage"
mkdir -p "$stage/include" "$stage/lib"
cp include/hwinfo.h "$stage/include/"

case "$target" in
  *windows*)
    cp "$src/hwinfo.dll" "$stage/lib/"
    found=0
    for lib in "$src/hwinfo.dll.lib" "$src/hwinfo.lib"; do
      if [[ -f "$lib" ]]; then
        cp "$lib" "$stage/lib/"
        found=1
      fi
    done
    if [[ "$found" != 1 ]]; then
      echo "missing Windows import library in $src" >&2
      exit 1
    fi
    ;;
  *apple*)
    cp "$src/libhwinfo.dylib" "$stage/lib/"
    ;;
  *linux*)
    cp "$src/libhwinfo.so" "$stage/lib/"
    ;;
  *)
    echo "unknown target: $target" >&2
    exit 1
    ;;
esac

tar -C dist -czf "hwinfo-${target}.tar.gz" "$target"
