#!/usr/bin/env python3
"""Embed the canonical About BMP on every platform (standard library only)."""
import sys
from pathlib import Path


def embed(source, destination):
    data = source.read_bytes()
    if len(data) < 54 or data[:2] != b"BM":
        raise ValueError(f"Invalid BMP: {source}")
    rows = [", ".join(f"0x{byte:02x}" for byte in data[i:i + 20])
            for i in range(0, len(data), 20)]
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        "// Generated from src/bitmaps/assdraw3.bmp. Do not edit.\n"
        "#pragma once\nnamespace AssdrawBitmapData {\n"
        "inline constexpr unsigned char About[] = {\n"
        + ",\n".join(rows) + "\n};\n}\n", encoding="utf-8")


if __name__ == "__main__":
    embed(Path(sys.argv[1]), Path(sys.argv[2]))
