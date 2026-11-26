#!/usr/bin/env python3
"""Summarizes a VIctory-macOS run log: in-world frame time and its parts.

Reads a run directory's stderr.txt (the host's frame lines) and reports the
medians over the frames after the world's first one, so two runs compare
like for like. Usage: summarize-run.py <log-dir> [<log-dir> ...]
"""

import re
import statistics
import sys

FIELDS = [
    ("ms", r" ([\d.]+) ms/frame"),
    ("fps", r" ([\d.]+) fps,"),
    ("prep", r"\(([\d.]+) preparing"),
    ("copy_ms", r"copies ([\d.]+) ms"),
    ("copy_mb", r"copies [\d.]+ ms ([\d.]+) MB"),
    ("gpu", r"([\d.]+) on the GPU"),
    ("back", r"([\d.]+) copying back"),
    ("draws", r"([\d.]+) draws/frame"),
    ("rec", r"; ([\d.]+) (?:recording|in \d+ draw calls)"),
    ("bind", r"([\d.]+) binding"),
    ("tables", r"tables ([\d.]+)"),
    ("values", r"values ([\d.]+)"),
    ("heap", r"heap ([\d.]+)"),
    ("dispatches", r"([\d.]+) in \d+ dispatches"),
]


def number(pattern, line):
    m = re.search(pattern, line)
    return float(m.group(1)) if m else None


def summarize(path):
    world = None
    rows = []
    with open(path, "r", errors="ignore") as log:
        for line in log:
            line = re.sub(r"\x1b\[[0-9;]*m", "", line)
            m = re.search(r"^(\S+) .*world's first frame", line)
            if m:
                world = m.group(1)
            if "ms/frame" not in line or "fps," not in line:
                continue
            row = {"t": line.split()[0]}
            for name, pattern in FIELDS:
                row[name] = number(pattern, line)
            rows.append(row)
    sel = [
        r
        for r in rows
        if world
        and r["t"] > world
        and (r["draws"] or 0) > 1000
    ]
    print(f"{path}")
    print(f"  world at {world}, in-world frame lines: {len(sel)}")
    if not sel:
        return
    parts = []
    for name, _ in FIELDS:
        values = [r[name] for r in sel if r[name] is not None]
        if not values:
            continue
        parts.append(f"{name} {statistics.median(values):.1f}")
    print("  median: " + ", ".join(parts))


if __name__ == "__main__":
    for directory in sys.argv[1:]:
        summarize(directory.rstrip("/") + "/stderr.txt")
