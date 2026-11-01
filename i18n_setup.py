#!/usr/bin/env python3
"""Compatibility entry point for catalog maintenance; C++ is never rewritten.

Run without arguments to validate catalogs. Use --update to extract/merge source
messages (GNU gettext required), or --compile-one PO MO to compile a catalog.
"""
from pathlib import Path
import runpy

if __name__ == "__main__":
    runpy.run_path(str(Path(__file__).resolve().parent / "tools/translations.py"), run_name="__main__")
