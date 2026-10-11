#!/usr/bin/env python3
"""Keep native conditional compilation inside markview-platform."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
CONDITIONS = re.compile(r"#\s*!?\s*\[\s*cfg(?:_attr)?\s*\(|\bcfg!\s*\(")
ALLOWED = {"test", "debug_assertions", "not", "all", "any", "(", ")", ","}
LINK_ATTRIBUTE = 'not(debug_assertions), windows_subsystem = "windows"'


def violations(source, main=False):
    for match in CONDITIONS.finditer(source):
        start = match.end()
        end = start
        depth = 1
        while end < len(source) and depth:
            depth += (source[end] == "(") - (source[end] == ")")
            end += 1
        condition = re.sub(r"\s+", " ", source[start:end - 1]).strip()
        tokens = set(re.findall(r"[A-Za-z_]\w*|[^\s]", condition))
        if tokens <= ALLOWED or (main and condition == LINK_ATTRIBUTE):
            continue
        yield source.count("\n", 0, match.start()) + 1, condition


def main():
    failures = []
    for directory in ("src", "tests"):
        for path in sorted((ROOT / directory).rglob("*.rs")):
            for line, condition in violations(path.read_text(), path == ROOT / "src/main.rs"):
                failures.append(f"{path.relative_to(ROOT)}:{line}: move conditional `{condition}` into markview-platform")
    for failure in failures:
        print(failure)
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
