#!/usr/bin/env python3
"""Export the original illustrated manual after extracting docs/ASSDraw3.chm.

Run once with: 7z x docs/ASSDraw3.chm -o/tmp/assdraw-manual
              python3 tools/export_manual.py /tmp/assdraw-manual
The exported HTML/images are committed; normal builds do not need 7-Zip.
"""

import argparse
import hashlib
import html
import json
from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parents[1]


def export(source):
    destination = ROOT / "docs/help/original"
    destination.mkdir(parents=True, exist_ok=True)
    toc = (source / "Table of Contents.hhc").read_text(encoding="cp1252")
    entries = []
    for block in re.findall(r'<object type="text/sitemap">(.*?)</object>', toc, re.S):
        params = dict(re.findall(r'<param name="([^"]+)" value="([^"]+)">', block))
        entries.append({key: html.unescape(value) for key, value in params.items()})
    names = {
        entry["Local"]: re.sub(r"[^a-z0-9]+", "-", entry["Name"].lower()).strip("-") + ".html"
        for entry in entries if "Local" in entry
    }
    style = ('<meta name="viewport" content="width=device-width,initial-scale=1">'
             '<style>body{max-width:1000px;margin:2rem auto;padding:0 1rem}'
             'img{max-width:100%;height:auto}.manual-nav{margin-bottom:1.5rem;'
             'font:16px/1.6 system-ui,sans-serif}</style>')
    nav = ('<nav class="manual-nav"><a href="../index.html">ASSDraw3 3.1 guide</a>'
           ' / <a href="index.html">Original manual contents</a></nav>')
    for old_name, new_name in names.items():
        content = (source / old_name).read_text(encoding="cp1252")
        content = content.replace("charset=Windows-1252", "charset=UTF-8")
        # This old CHM helper constructs Windows file URLs; normal relative
        # links work offline in browsers without JavaScript or ActiveX.
        content = re.sub(r"<script\b[^>]*>.*?</script>\s*", "", content, flags=re.S | re.I)
        for old_link, new_link in names.items():
            content = content.replace(old_link, new_link)
        content = content.replace("</head>", style + "</head>")
        content = content.replace("<body>", "<body>\n" + nav, 1)
        (destination / new_name).write_text(content, encoding="utf-8", newline="\n")
    for image in sorted(source.glob("img_*.jpg")):
        shutil.copyfile(image, destination / image.name)

    contents = ["<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">",
                "<title>ASSDraw3 original illustrated manual</title>", style,
                "<body><nav><a href=\"../index.html\">ASSDraw3 3.1 guide</a></nav>",
                "<h1>Original illustrated manual</h1>",
                "<p>Original ASSDraw3 documentation by ai-chan (2007), including all illustrations. "
                "Some screenshots and instructions describe the original version. "
                "See the 3.1 guide for coloring and language support.</p>"]
    for entry in entries:
        label = html.escape(entry["Name"])
        if "Local" in entry:
            contents.append(f'<p><a href="{names[entry["Local"]]}">{label}</a></p>')
        else:
            contents.append(f"<h2>{label}</h2>")
    contents.append("</body></html>\n")
    (destination / "index.html").write_text("\n".join(contents), encoding="utf-8")
    files = sorted(destination.glob("*.html")) + sorted(destination.glob("*.jpg"))
    manifest = {
        "source": "docs/ASSDraw3.chm",
        "source_sha256": hashlib.sha256((ROOT / "docs/ASSDraw3.chm").read_bytes()).hexdigest(),
        "topics": len(names),
        "images": len(list(destination.glob("*.jpg"))),
        "files": {file.name: hashlib.sha256(file.read_bytes()).hexdigest() for file in files},
    }
    (destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f'Exported {manifest["topics"]} topics and {manifest["images"]} illustrations.')


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("extracted_directory", type=Path)
    export(parser.parse_args().extracted_directory)
