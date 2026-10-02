#!/usr/bin/env bash
set -euo pipefail

original_java_home="${JAVA_HOME:?actions/setup-java must provide JAVA_HOME}"
smoke_root="$(mktemp -d)"
cp target/release/jsh "$smoke_root/jsh"
mkdir -p "$smoke_root/home"
python3 - "$smoke_root/config.json" "$original_java_home" <<'PY'
import json
import pathlib
import sys

pathlib.Path(sys.argv[1]).write_text(json.dumps({
    "current_jdk": None,
    "jdks": {},
    "download_dir": "downloads",
    "scan_dirs": [sys.argv[2]],
}), encoding="utf-8")
PY

export HOME="$smoke_root/home"
export SHELL=/bin/bash
export NO_COLOR=1

"$smoke_root/jsh" list > "$smoke_root/list-before.txt"
grep -Fq 'JDK 17' "$smoke_root/list-before.txt"

"$smoke_root/jsh" download 21
"$smoke_root/jsh" list > "$smoke_root/list-after.txt"
grep -Fq 'JDK 17' "$smoke_root/list-after.txt"
grep -Fq 'JDK 21' "$smoke_root/list-after.txt"

"$smoke_root/jsh" use 17
source "$HOME/.bashrc"
java -version 2> "$smoke_root/java-17.txt"
grep -Eq 'version "17([.+"]|$)' "$smoke_root/java-17.txt"

"$smoke_root/jsh" use 21
java -version 2> "$smoke_root/java-21.txt"
grep -Eq 'version "21([.+"]|$)' "$smoke_root/java-21.txt"
javac -version > "$smoke_root/javac-21.txt" 2>&1
grep -Eq '^javac 21([.+]|$)' "$smoke_root/javac-21.txt"
"$smoke_root/jsh" current > "$smoke_root/current.txt"
grep -Fq 'JDK 21' "$smoke_root/current.txt"
if grep -Fq 'java on PATH resolves to' "$smoke_root/current.txt"; then
    cat "$smoke_root/current.txt"
    exit 1
fi

echo "Real JDK download, list, and open-shell switch passed on $(uname -s) $(uname -m)."
