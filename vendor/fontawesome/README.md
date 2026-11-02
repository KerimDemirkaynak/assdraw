# Font Awesome Free toolbar icons

Original SVG files from Font Awesome Free **7.2.0**, commit
`337dd2045d5621ce0f8567c33c256f3dedeed55d`:
https://github.com/FortAwesome/Font-Awesome/tree/7.2.0/svgs

Copyright 2026 Fonticons, Inc. https://fontawesome.com
Icons are licensed under CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
The original license and attribution comments are retained.

`tools/embed_icons.py` adds uniform square padding and embeds these files at
build time. `src/icons.cpp` replaces `currentColor` with the native text colour
when creating SVG bitmap bundles. Paths are unchanged. No icon font, network
access, or external runtime files are needed.
