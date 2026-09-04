# Fonts for the manual

`typst compile` runs with `--ignore-system-fonts`, so that the rendered manual
comes out byte-identical wherever it is built rather than picking up whichever
faces the machine happens to have installed. Everything the manual sets in type
is covered by the faces Typst embeds, with one exception: `manual.typ` shows a
Han Xin sample payload, and no embedded face carries a CJK ideograph.

`NotoSansSC-Regular.subset.otf` closes that gap and nothing else. It holds the
two ideographs the manual uses and weighs under 4 KB, where the full face is
around 8 MB. Typst falls back to it per glyph, so nothing has to name it.

## Provenance

A subset of Noto Sans SC Regular, version 2.004, copyright 2014-2021 Adobe,
licensed under the SIL Open Font License 1.1. The licence text is in `OFL.txt`
and is also recorded in the font's own name table.

Taken from
<https://github.com/notofonts/noto-cjk/raw/main/Sans/SubsetOTF/SC/NotoSansSC-Regular.otf>.

## Regenerating

Only needed when the manual starts using an ideograph that is not in the list
below. Built with fontTools 4.63.0:

```sh
pip install fonttools
python -m fontTools.subset NotoSansSC-Regular.otf \
  --unicodes=U+5168,U+6F04 \
  --name-IDs='*' \
  --output-file=dist/fonts/NotoSansSC-Regular.subset.otf
```

U+5168 and U+6F04 are the ideographs in the Han Xin payload on `manual.typ`.
`--name-IDs='*'` keeps the name table, and with it the attribution and the
licence the OFL asks to travel with the font.
