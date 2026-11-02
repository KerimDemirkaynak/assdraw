#!/usr/bin/env python3
"""Check that the offline manual's contents, illustrations and links are intact."""

import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]


class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.urls = []

    def handle_starttag(self, tag, attributes):
        for name, value in attributes:
            if name in ("href", "src") and value:
                self.urls.append(value)


def check_manual(directory):
    directory = Path(directory)
    manifest = json.loads((directory / "original/manifest.json").read_text(encoding="utf-8"))
    if manifest["topics"] != 22 or manifest["images"] != 14:
        raise ValueError("Incomplete original manual")
    chm = directory.parent / "ASSDraw3.chm"
    if hashlib.sha256(chm.read_bytes()).hexdigest() != manifest["source_sha256"]:
        raise ValueError("The original CHM manual differs from the HTML export's source")
    for name, digest in manifest["files"].items():
        path = directory / "original" / name
        content = path.read_bytes()
        # Git's Windows checkout may use CRLF. Preserve the integrity check
        # while accepting the same HTML text with either native newline form.
        if path.suffix.lower() == ".html":
            content = content.replace(b"\r\n", b"\n")
        if hashlib.sha256(content).hexdigest() != digest:
            raise ValueError(f"Missing or damaged manual content: {path}")
    for page in directory.rglob("*.html"):
        content = page.read_text(encoding="utf-8")
        parser = Links()
        parser.feed(content)
        if len(content) < 300 or "<body" not in content.lower():
            raise ValueError(f"Empty manual page: {page}")
        for url in parser.urls:
            parsed = urlsplit(url)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            target = (page.parent / unquote(parsed.path)).resolve()
            if not target.is_file():
                raise ValueError(f"Broken manual link in {page}: {url}")
    print("Offline manual: all 22 original topics, 14 illustrations, and local links verified.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", nargs="?", type=Path, default=ROOT / "docs/help")
    check_manual(parser.parse_args().directory)
