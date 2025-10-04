#!/usr/bin/env bash
for f in tests/*.rs; do
  opens=$(grep -o '{' "$f" | wc -l)
  closes=$(grep -o '}' "$f" | wc -l)
  if [ "$opens" -ne "$closes" ]; then
    echo "$f : {=$opens }=$closes"
  fi
done
