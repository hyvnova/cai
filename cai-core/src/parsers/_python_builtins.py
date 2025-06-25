from pathlib import Path
from typing import Iterable, Union
import sys

# Force UTF-8 and swallow unencodable chars instead of crashing
sys.stdout.reconfigure(encoding="utf-8", errors="backslashreplace")
sys.stderr.reconfigure(encoding="utf-8", errors="backslashreplace")

def replace_code(
    start: int,                    # 1-based line number of FIRST line to replace
    end: int,                      # 1-based line number of LAST  line to replace
    code: Union[str, Iterable[str]],
    file_path: str | Path = "demo.py",
) -> None:
    """
    Splice `code` into `file_path`, replacing the lines from `start` through `end`
    (inclusive). Works in-place, but writes atomically via a temp file.

    - `code` can be:
        • a raw string  → split on '\n'
        • any iterable  → each item is taken as a line
    - Blank-line expansion, AST magic, etc. is YOUR problem;
      this fn just blind-cuts the given range.
    """
    p = Path(file_path)

    # Read file (preserve newlines)
    lines = p.read_text().splitlines(keepends=True)

    # Normalise args
    if start < 1 or end < start or end > len(lines):
        raise IndexError(f"Bad range: {start=}, {end=}, file has {len(lines)} lines")

    if isinstance(code, str):
        new_block = [(ln + "\n") if not ln.endswith("\n") else ln
                     for ln in code.splitlines()]
    else:  # assume iterable
        new_block = [(ln + "\n") if not ln.endswith("\n") else ln
                     for ln in code]

    # Splice (convert to 0-based slice)
    lines[start - 1 : end] = new_block

    # Atomic write
    tmp = p.with_suffix(p.suffix + ".tmp")
    tmp.write_text("".join(lines))
    tmp.replace(p)
