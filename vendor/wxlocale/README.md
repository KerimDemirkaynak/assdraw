# Standard wxWidgets control translations

Source: [wxWidgets v3.3.1 locale catalogs](https://github.com/wxWidgets/wxWidgets/tree/v3.3.1/locale).
The unmodified source PO files and their contributor notices are retained under
`source/`. The runtime `wxstd.mo` files were compiled with GNU `msgfmt`:

```sh
msgfmt --check-format -o LANGUAGE/LC_MESSAGES/wxstd.mo source/LANGUAGE.po
```

`catalogs.json` records source and compiled SHA-256 values. English uses built-in
source messages. These catalogs translate standard wxWidgets dialogs, property
editors, and buttons; application text remains in the separate `assdraw` domain.
The original wxWindows license and LGPL text are included unchanged.
