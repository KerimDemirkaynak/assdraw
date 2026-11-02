#!/usr/bin/env python3
"""Maintain and compile ASSDraw catalogs; never rewrite C++ or build files."""
import argparse
import ast
import gettext
import re
import shutil
import struct
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LANGUAGES = ("tr", "fr", "de", "es", "zh_TW", "vi")


def read_po(path):
    messages, entry, field, flags = {}, {}, None, set()

    def finish():
        nonlocal entry, field, flags
        if "msgid" in entry:
            key = entry["msgid"]
            if "msgctxt" in entry:
                key = entry["msgctxt"] + "\x04" + key
            if key in messages:
                raise ValueError(f"{path}: duplicate message: {key!r}")
            if "fuzzy" in flags and key:
                raise ValueError(f"{path}: fuzzy message: {key!r}")
            messages[key] = entry.get("msgstr", "")
        entry, field, flags = {}, None, set()

    for number, raw in enumerate(Path(path).read_text(encoding="utf-8-sig").splitlines(), 1):
        line = raw.strip()
        if not line:
            finish()
        elif line.startswith("#,"):
            flags.update(item.strip() for item in line[2:].split(","))
        elif line.startswith("#"):
            continue
        else:
            match = re.fullmatch(r'(msgctxt|msgid|msgstr)\s+(".*")', line)
            if match:
                if match[1] == "msgid" and "msgstr" in entry:
                    finish()
                field = match[1]
                value = ast.literal_eval(match[2])
                if not isinstance(value, str):
                    raise ValueError(f"{path}:{number}: expected a string")
                entry[field] = value
            elif line.startswith('"') and field:
                entry[field] += ast.literal_eval(line)
            else:
                raise ValueError(f"{path}:{number}: unsupported PO syntax")
    finish()
    return messages


def compile_catalog(source, destination):
    messages = read_po(source)
    if "charset=UTF-8" not in messages.get("", ""):
        raise ValueError(f"{source}: UTF-8 header is required")
    if any(not value for key, value in messages.items() if key):
        raise ValueError(f"{source}: untranslated message")
    items = sorted((key.encode("utf-8"), value.encode("utf-8")) for key, value in messages.items())
    count = len(items)
    original_offset = 28 + count * 16
    originals = b"".join(key + b"\0" for key, value in items)
    translations = b"".join(value + b"\0" for key, value in items)
    left, right, offset = [], [], original_offset
    for key, value in items:
        left.append((len(key), offset))
        offset += len(key) + 1
    offset = original_offset + len(originals)
    for key, value in items:
        right.append((len(value), offset))
        offset += len(value) + 1
    header = struct.pack("<7I", 0x950412DE, 0, count, 28, 28 + count * 8, 0, 0)
    tables = b"".join(struct.pack("<2I", *item) for item in left + right)
    destination = Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(header + tables + originals + translations)
    with destination.open("rb") as stream:
        compiled = gettext.GNUTranslations(stream)
    for key, value in messages.items():
        if key and "\x04" not in key and compiled.gettext(key) != value:
            raise ValueError(f"{source}: MO round trip failed: {key!r}")


def extract(destination):
    program = shutil.which("xgettext")
    if not program:
        raise ValueError("Install GNU gettext to update the source catalog.")
    sources = sorted(str(p.relative_to(ROOT)) for p in (ROOT / "src").rglob("*")
                     if p.suffix in (".cpp", ".hpp", ".h"))
    subprocess.run([program, "--language=C++", "--from-code=UTF-8", "--keyword=_",
                    "--keyword=wxTRANSLATE", "--keyword=wxGetTranslation:1",
                    "--sort-output", "--no-wrap", "--package-name=ASSDraw3",
                    "--msgid-bugs-address=https://github.com/KerimDemirkaynak/assdraw/issues",
                    "--package-version=3.1", "-o", str(destination), *sources], cwd=ROOT, check=True)


PRINTF = re.compile(r"%(?!%)(?:\d+\$)?[-+ #0]*(?:\d+|\*)?(?:\.(?:\d+|\*))?(?:hh|ll|[hljztL])?[diuoxXfFeEgGaAcsp]")


def check():
    template = read_po(ROOT / "po/assdraw.pot")
    expected = set(template) - {""}
    if (ROOT / "po/en.po").exists():
        raise ValueError("English is the source language; po/en.po must not exist.")
    if shutil.which("xgettext"):
        with tempfile.TemporaryDirectory() as directory:
            pot = Path(directory) / "current.pot"
            extract(pot)
            actual = set(read_po(pot)) - {""}
        if actual != expected:
            raise ValueError(f"Source catalog is stale: added={sorted(actual-expected)!r}; removed={sorted(expected-actual)!r}")
    for language in LANGUAGES:
        path = ROOT / f"po/{language}.po"
        messages = read_po(path)
        if set(messages) - {""} != expected:
            raise ValueError(f"{path}: catalog entries differ from the source template")
        for key in expected:
            value = messages[key]
            if not value.strip():
                raise ValueError(f"{path}: untranslated message: {key!r}")
            if PRINTF.findall(key) != PRINTF.findall(value):
                raise ValueError(f"{path}: format placeholders differ: {key!r}")
            if key.split("\t")[1:] != value.split("\t")[1:]:
                raise ValueError(f"{path}: shortcut changed: {key!r}")
        with tempfile.TemporaryDirectory() as directory:
            compile_catalog(path, Path(directory) / "assdraw.mo")
        with (ROOT / f"vendor/wxlocale/{language}/LC_MESSAGES/wxstd.mo").open("rb") as stream:
            gettext.GNUTranslations(stream)
        print(f"{language}: {len(expected)} translated messages; format strings and MO round trip verified")


def check_package(directory):
    from check_manual import check_manual
    directory = Path(directory)
    for language in LANGUAGES:
        path = directory / f"locale/{language}/LC_MESSAGES/assdraw.mo"
        with path.open("rb") as stream:
            catalog = gettext.GNUTranslations(stream)
        with (directory / f"locale/{language}/LC_MESSAGES/wxstd.mo").open("rb") as stream:
            gettext.GNUTranslations(stream)
        if catalog.gettext("Pan background") == "Pan background":
            raise ValueError(f"{path}: application translations not loaded")
    chm = directory / "ASSDraw3.chm"
    if chm.read_bytes()[:4] != b"ITSF":
        raise ValueError("The original CHM manual is missing or invalid")
    if not (directory / "help/index.html").is_file():
        raise ValueError("The portable HTML guide is missing")
    check_manual(directory / "help")
    print("Packaged application includes all six language catalogs and both manuals.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--check", action="store_true")
    group.add_argument("--update", action="store_true")
    group.add_argument("--compile-one", nargs=2, metavar=("PO", "MO"))
    group.add_argument("--check-package", metavar="DIRECTORY")
    args = parser.parse_args()
    if args.check_package:
        check_package(args.check_package)
    elif args.compile_one:
        compile_catalog(*args.compile_one)
    elif args.update:
        extract(ROOT / "po/assdraw.pot")
        merge = shutil.which("msgmerge")
        if not merge:
            parser.error("Install GNU gettext to merge catalogs.")
        for language in LANGUAGES:
            subprocess.run([merge, "--update", "--backup=off", "--no-fuzzy-matching", "--no-wrap",
                            str(ROOT / f"po/{language}.po"), str(ROOT / "po/assdraw.pot")], check=True)
    else:
        check()


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error))
