# Changelog

## Unreleased

### Features

- Add `extension: [grid_table: true]` for Pandoc-style grid tables, with the `GridTable`, `GridTableRow`, and `GridTableCell` AST nodes and `GridTableColumn` for its columns, and `render: [grid_table_widths: true]` to write their column widths ([Comrak #820](https://github.com/kivikakk/comrak/pull/820)).
- Add `extension: [math_dollars_block: true]` for display math between `$$` lines, with the `MathBlock` AST node. Needs `math_dollars: true` too ([Comrak #931](https://github.com/kivikakk/comrak/pull/931)).
- Add `extension: [ofm_block_ids: true]` for Obsidian block identifiers (`^name`), with the `BlockId` AST node ([Comrak #900](https://github.com/kivikakk/comrak/pull/900)).
- Add `render: [attributes: true]`. With `render: [unsafe: true]`, it renders parsed attributes on headings, code blocks, and inline code too, not only on links and images. Code block attributes go on the `<pre>`, including the one Lumis writes ([Comrak #903](https://github.com/kivikakk/comrak/pull/903)).
- `document_to_commonmark/2` writes attributes back when their extension is enabled ([Comrak #939](https://github.com/kivikakk/comrak/pull/939)).

### Security

- Fix quadratic time on escaped pipes in table cells, failing autolink domains, and non-literal emails ([GHSA-7f8r-4rh5-vr5h](https://github.com/kivikakk/comrak/security/advisories/GHSA-7f8r-4rh5-vr5h)), plus other quadratic cases in block IDs, attributes, alerts, multiline block quotes, strikethrough and highlight runs, unclosed inline footnotes, LaTeX math, and HEEx directives, and in CommonMark output of grid tables and deep nesting ([Comrak #948](https://github.com/kivikakk/comrak/pull/948), [#949](https://github.com/kivikakk/comrak/pull/949)).
- Limit output amplification: lists, footnote definitions, description items, and fenced containers nest at most 100 deep, after which markers are read as text; autocompleted table cells share one allowance per document; CommonMark output writes lazy continuation lines past 64-byte container prefixes and falls back to cell contents for grid tables over its area allowance ([Comrak #948](https://github.com/kivikakk/comrak/pull/948)).

### Changes

Comrak 1.x has written specs for most extensions, and some parsing changed with them:

- Footnotes: a label defined twice uses its first definition instead of rendering both; labels match as written, so `[^&#97;]` no longer matches `[^a]`; definitions nested in definitions are found; footnotes are numbered in reading order; a blank line inside a container no longer ends a definition. `FootnoteReference.texts` holds the label as written, such as `[{"^1", 2}]` ([Comrak #935](https://github.com/kivikakk/comrak/pull/935)).
- Attributes: keys may start with `_` or `:`, values may be unquoted (`{k=a/b}`), entities in values are decoded, `id=` and `class=` set the id and add classes, and the last of repeated keys wins. Consecutive attribute blocks merge ([Comrak #939](https://github.com/kivikakk/comrak/pull/939)).
- Attributes that MDExNative or Comrak writes itself on a link or image, such as `href`, `src`, `alt`, `title`, or `data-sourcepos`, take precedence over attributes with the same name, which are no longer written twice. Keys that are not valid attribute names are dropped when a document is converted, so Comrak's HTML and CommonMark output never includes them.
- Wikilinks: an unclosed `[` no longer stops a following wikilink from parsing, wikilinks parse in image descriptions, and line endings in destinations become spaces ([Comrak #936](https://github.com/kivikakk/comrak/pull/936)).
- Alerts: `>[!note]`, and up to three spaces or a tab after `>`, start an alert ([Comrak #938](https://github.com/kivikakk/comrak/pull/938)).
- Task lists: markers must appear literally and be followed by a space, tab, or line end, so `[x]*a*` is no longer a task item ([Comrak #923](https://github.com/kivikakk/comrak/pull/923)).
- Tables: backslashes before a pipe pair correctly, so `c\\|` ends the cell after `c\`. Autolinks: emails containing entities no longer link, and `_www.example.com_` does. Strikethrough: a mismatched `~` no longer stops inline parsing ([Comrak #926](https://github.com/kivikakk/comrak/pull/926), [#927](https://github.com/kivikakk/comrak/pull/927), [#928](https://github.com/kivikakk/comrak/pull/928)).
- `\r\n` and `\r` are normalized to `\n` in block content, so carriage returns no longer reach literals ([Comrak #915](https://github.com/kivikakk/comrak/pull/915)).
- Reference links expand up to at least 100 KB or the document's size, whichever is larger, instead of at most the smaller ([Comrak #905](https://github.com/kivikakk/comrak/pull/905)).
- Many source positions are corrected, including the end of a list's last item, inline footnotes, and unclosed fenced code, multiline block quotes, block directives, and alerts ([Comrak #901](https://github.com/kivikakk/comrak/pull/901), [#908](https://github.com/kivikakk/comrak/pull/908)).
- Math: an escaped backslash no longer escapes the closer after it, code math (`` $`…`$ ``) needs matching backtick runs, dollar math keeps line endings, and ```` ```math ```` blocks are math only with `math_code: true` ([Comrak #941](https://github.com/kivikakk/comrak/pull/941)). Math code blocks get `data-meta` and attributes like other code blocks ([Comrak #942](https://github.com/kivikakk/comrak/pull/942)).
- Description lists: `:` follows list marker rules, so it can stand alone and its content can start on the next line; a blank line between details makes the item loose. OFM block IDs label the `<dt>` or `<dd>` they end instead of the `<dl>` ([Comrak #945](https://github.com/kivikakk/comrak/pull/945)).
- Superscript and subscript: superscript opens inside brackets, punctuation no longer affects `^` and `~` flanking, carets pair one at a time, and subscript no longer changes strikethrough flanking ([Comrak #946](https://github.com/kivikakk/comrak/pull/946)).
- Header IDs never collide with explicit attribute IDs, footnote IDs, or OFM block IDs under `header_id_prefix`, and are never empty; an empty heading gets `-1` ([Comrak #944](https://github.com/kivikakk/comrak/pull/944)).
- XML writes table header rows as `<table_header>`, escaped tags as `<escaped_tag tag="~~">`, `tight` on `description_item`, and escapes code block info strings ([Comrak #926](https://github.com/kivikakk/comrak/pull/926), [#945](https://github.com/kivikakk/comrak/pull/945), [#946](https://github.com/kivikakk/comrak/pull/946), [#949](https://github.com/kivikakk/comrak/pull/949)).

### Dependencies

- Update Comrak from 0.59.0 to 1.2.0.

### Compatibility

- Downstream AST adapters (including MDEx's Comrak converter) must handle `BlockId`, `GridTable`, `GridTableRow`, `GridTableCell`, `GridTableColumn`, and `MathBlock`. They appear only when their extensions are enabled.

## [0.3.3](https://github.com/leandrocp/mdex_native/compare/v0.3.2...v0.3.3) (2026-11-23)

### Bug Fixes

- **nif:** Ship precompiled Linux NIFs without static TLS in [\#161](https://github.com/leandrocp/mdex_native/pull/161)

### Dependencies

- Update Lumis core and WASM runtime in [\#152](https://github.com/leandrocp/mdex_native/pull/152) and [\#159](https://github.com/leandrocp/mdex_native/pull/159)

## [0.3.2](https://github.com/leandrocp/mdex_native/compare/v0.3.1...v0.3.2) (2026-11-14)

### Bug Fixes

- Build linux-gnu NIFs against glibc 2.28 in [\#148](https://github.com/leandrocp/mdex_native/pull/148)

### Dependencies

- **deps:** Update Lumis core and WASM runtime in [\#139](https://github.com/leandrocp/mdex_native/pull/139)

## [0.3.1](https://github.com/leandrocp/mdex_native/compare/v0.3.0...v0.3.1) (2026-10-31)

### Features

- Add `budget: [parse_memory: bytes]` to cap the heap a parse may use. By @ericmj in [\#129](https://github.com/leandrocp/mdex_native/pull/129)
- Highlight inline code with Lumis via a trailing `{:lang}` marker or a `language-*` class. Disable with `syntax_highlight: [inline_code: false]`. In [\#132](https://github.com/leandrocp/mdex_native/pull/132)
- Highlight inline code with Syntect too. In [\#133](https://github.com/leandrocp/mdex_native/pull/133)

### Bug Fixes

- An unknown Syntect theme name no longer panics. In [\#133](https://github.com/leandrocp/mdex_native/pull/133)
- Lumis diff fences keep the line background across the marker and injected tokens. In [\#135](https://github.com/leandrocp/mdex_native/pull/135)

## [0.3.0](https://github.com/leandrocp/mdex_native/compare/v0.2.14...v0.3.0) (2026-10-29)

### Breaking Changes

- `MDExNative.Comrak.parse_document/2`, `markdown_to_html/2`, `markdown_to_xml/2`, `document_to_html/2`, `document_to_xml/2`, and `document_to_commonmark/2` now return `{:ok, result}` or `{:error, exception}`. To keep the old behavior, call the new bang variants. In [\#123](https://github.com/leandrocp/mdex_native/pull/123)

### Features

- Add `extension: [ofm_comments: true]` and the `CommentInline` / `MultilineComment` AST nodes. Comments are omitted from HTML but preserved in CommonMark and XML.
- Add `extension: [ofm_callouts: true]` and the `Callout` / `CalloutTitle` AST nodes, including metadata, Markdown titles, and fold state.
- Preserve wikilink embeds (`![[url]]`) in `WikiLink.embed` and CommonMark/XML output. HTML renders embeds as links. Set `extension: [wikilinks_ignore_embeds: true]` to leave embeds as literal text.
- Add a top-level `budget: [parse_time: milliseconds]` option that bounds Comrak's parse in `parse_document/2`, `markdown_to_html/2`, and `markdown_to_xml/2`. Running out returns `{:error, %MDExNative.ParseTimeExceededError{}}` without partial output; the bang variants raise it. Omitting `budget`, or setting `parse_time` to `nil` or `0`, means no limit. In [\#122](https://github.com/leandrocp/mdex_native/pull/122)

### Security

- Include Comrak's fix for quadratic HTML table-cell rendering (GHSA-j7hq-qg5v-47jq).

### Dependencies

- Upgrade Comrak to 0.59 in [\#119](https://github.com/leandrocp/mdex_native/pull/119)

### Compatibility

- Downstream AST adapters (including MDEx's Comrak converter) must handle `Callout`, `CalloutTitle`, `CommentInline`, and `MultilineComment`, and preserve the new `WikiLink.embed` field. Wikilink XML now includes `embed="true"` or `embed="false"`.

## [0.2.14](https://github.com/leandrocp/mdex_native/compare/v0.2.13...v0.2.14) (2026-10-28)

### Bug Fixes

- Fall back to plain text when Lumis highlighting fails in [\#114](https://github.com/leandrocp/mdex_native/pull/114)

### Dependencies

- Update Lumis core to 6.0 in [\#118](https://github.com/leandrocp/mdex_native/pull/118)

## [0.2.13](https://github.com/leandrocp/mdex_native/compare/v0.2.12...v0.2.13) (2026-10-23)

### Features

- Lumis highlight doesn't require the :lumis package in [\#88](https://github.com/leandrocp/mdex_native/pull/88)

### Bug Fixes

- Cache Syntect adapters by theme across renders (#102)


## [0.2.12](https://github.com/leandrocp/mdex_native/compare/v0.2.11...v0.2.12) (2026-10-19)

### Bug Fixes

- **nif:** Bump MSRV and Rustler in [\#101](https://github.com/leandrocp/mdex_native/pull/101)

### Dependencies

- Update Comrak to 0.56.0 in [\#94](https://github.com/leandrocp/mdex_native/pull/94)

## [0.2.11](https://github.com/leandrocp/mdex_native/compare/v0.2.10...v0.2.11) (2026-10-04)

### Bug Fixes

- Forward Lumis highlighting budgets in [\#89](https://github.com/leandrocp/mdex_native/pull/89)

### Performance Improvements

- Strip symbols from the release NIF in [\#87](https://github.com/leandrocp/mdex_native/pull/87)

### Documentation

- Add an architecture overview in [\#84](https://github.com/leandrocp/mdex_native/pull/84)

## [0.2.10](https://github.com/leandrocp/mdex_native/compare/v0.2.9...v0.2.10) (2026-09-29)

### Features

- Render node attributes in HTML output by @leandrocp in [#76](https://github.com/leandrocp/mdex_native/pull/76)
- Update Lumis to 0.10 by @leandrocp in [#82](https://github.com/leandrocp/mdex_native/pull/82)

### Bug Fixes

- Configure the Lumis store on first use, not only at startup by @leandrocp in [#79](https://github.com/leandrocp/mdex_native/pull/79)
- Cache compiled parsers under the Lumis data directory by @leandrocp in [#80](https://github.com/leandrocp/mdex_native/pull/80)

### Documentation

- Link the upstream comrak source used in formatter by @leandrocp in [#78](https://github.com/leandrocp/mdex_native/pull/78)

## [0.2.9](https://github.com/leandrocp/mdex_native/compare/v0.2.8...v0.2.9) (2026-09-24)

### Features

- Update comrak to 0.55.0 by @leandrocp in [#65](https://github.com/leandrocp/mdex_native/pull/65)
- **perf:** Render Markdown faster by @leandrocp in [#71](https://github.com/leandrocp/mdex_native/pull/71)
- Lumis WASM runtime and parsers from Hex by @leandrocp in [#59](https://github.com/leandrocp/mdex_native/pull/59)

### Bug Fixes

- Declare children on Escaped AST nodes by @leandrocp in [#70](https://github.com/leandrocp/mdex_native/pull/70)

### Documentation

- Clarify precompiled artifact compatibility by @leandrocp in [#67](https://github.com/leandrocp/mdex_native/pull/67)
- Lumis 0.9 by @leandrocp in [#75](https://github.com/leandrocp/mdex_native/pull/75)

## [0.2.8](https://github.com/leandrocp/mdex_native/compare/v0.2.7...v0.2.8) (2026-08-13)

### Features

- Update lumis v0.13 by @leandrocp in [#53](https://github.com/leandrocp/mdex_native/pull/53)

## 0.2.7 (2026-08-03)

<!-- Release notes generated using configuration in .github/release.yml at main -->

## What's Changed
* docs: Document Comrak AST modules by @leandrocp in https://github.com/leandrocp/mdex_native/pull/44
* fix: reset Lumis state between code fences by @lobo-tuerto in https://github.com/leandrocp/mdex_native/pull/47

## New Contributors
* @lobo-tuerto made their first contribution in https://github.com/leandrocp/mdex_native/pull/47

**Full Changelog**: https://github.com/leandrocp/mdex_native/compare/v0.2.6...v0.2.7

## [0.2.6](https://github.com/leandrocp/mdex_native/compare/v0.2.5...v0.2.6) (2026-07-17)


### Features

* update comrak ([#42](https://github.com/leandrocp/mdex_native/issues/42)) ([fb5462a](https://github.com/leandrocp/mdex_native/commit/fb5462a4c6d68e3da40f51f6991fb47e4d63c213))

## [0.2.5](https://github.com/leandrocp/mdex_native/compare/v0.2.4...v0.2.5) (2026-07-08)


### Features

* update comrak v0.53 ([#40](https://github.com/leandrocp/mdex_native/issues/40)) ([7e239ba](https://github.com/leandrocp/mdex_native/commit/7e239ba01ca72190119a5c473ef0f13add150fae))
* Reduce comrak features to only ones used by @josevalim

### Bug Fixes

* bump minimum rustler_precompiled ([#37](https://github.com/leandrocp/mdex_native/issues/37)) ([36bd51c](https://github.com/leandrocp/mdex_native/commit/36bd51c809b9107ba2e2eb373b7f024064ba1f2d)), closes [#33](https://github.com/leandrocp/mdex_native/issues/33)
* render attrs on pre tag for html multi themes formatter ([#39](https://github.com/leandrocp/mdex_native/issues/39)) ([05204c9](https://github.com/leandrocp/mdex_native/commit/05204c9b3e532ca555746e45a5bddd7bcb1d379d))

## [0.2.4](https://github.com/leandrocp/mdex_native/compare/v0.2.3...v0.2.4) (2026-07-03)


### Bug Fixes

* preserve curly braces ([#36](https://github.com/leandrocp/mdex_native/issues/36)) ([7cd5eae](https://github.com/leandrocp/mdex_native/commit/7cd5eae43fdd515e23fdc622d6c3dcba03a8d032))


### Documentation

* cloudflare source example ([917a54f](https://github.com/leandrocp/mdex_native/commit/917a54f3c2e6c1dd503f132ba920f27ab25ed6e5))

## [0.2.3](https://github.com/leandrocp/mdex_native/compare/v0.2.2...v0.2.3) (2026-06-29)


### Features

* cloudflare mirror ([#28](https://github.com/leandrocp/mdex_native/issues/28)) ([f249b57](https://github.com/leandrocp/mdex_native/commit/f249b57329b6ee317bdb6e670ae446e2898402a5))


### Bug Fixes

* avoid leaking escaped tag literals ([cbd927f](https://github.com/leandrocp/mdex_native/commit/cbd927fb5061b488de8d90a8ef6df65718ca1fe6))
* **lumis:** sanitize decorator attributes ([7738ff4](https://github.com/leandrocp/mdex_native/commit/7738ff428a54058ea0a3293eb2e17b5f4c953e2d))
* **lumis:** unbounded Lumis highlight_lines ranges ([798a363](https://github.com/leandrocp/mdex_native/commit/798a363b4339f6f7162ec8437c4c9f9b5ae6fbf3))
* stack-safe comrak document conversion ([947696c](https://github.com/leandrocp/mdex_native/commit/947696c47bc22bea5dffc0f78c946fa6b70ce183))

## [0.2.2](https://github.com/leandrocp/mdex_native/compare/v0.2.1...v0.2.2) (2026-06-19)


### Features

* add `MDExNative.Comrak.dangerous_url?` ([#24](https://github.com/leandrocp/mdex_native/issues/24)) ([d05d02b](https://github.com/leandrocp/mdex_native/commit/d05d02bc954d5f869c2619c442f46da0bca70757))


### Documentation

* expose changelog ([487cb1c](https://github.com/leandrocp/mdex_native/commit/487cb1c6a32ba51b1fc9ee8cabe666bb46c73cfb))

## [0.2.1](https://github.com/leandrocp/mdex_native/compare/v0.2.0...v0.2.1) (2026-06-15)


### Features

* expose sanitization options ([#19](https://github.com/leandrocp/mdex_native/issues/19)) ([45b7d0f](https://github.com/leandrocp/mdex_native/commit/45b7d0f37e95cf49caa99eca6c4630b2dcb68b6e))

## [0.2.0](https://github.com/leandrocp/mdex_native/compare/v0.1.5...v0.2.0) (2026-06-12)


### ⚠ BREAKING CHANGES

* adopt the `l-` prefix on token class https://github.com/leandrocp/lumis/pull/952

### Features

* update lumis ([#17](https://github.com/leandrocp/mdex_native/issues/17)) ([b285d88](https://github.com/leandrocp/mdex_native/commit/b285d88530d17ae3d969ae191b05f27d02826a8d))

## [0.1.5](https://github.com/leandrocp/mdex_native/compare/v0.1.4...v0.1.5) (2026-06-08)


### Features

* choose syntax highlighter at compile-time ([#16](https://github.com/leandrocp/mdex_native/issues/16)) ([c4aecc2](https://github.com/leandrocp/mdex_native/commit/c4aecc2fa1288613f728a7f041a32d9669f9a67b))


### Documentation

* project description ([8a240a2](https://github.com/leandrocp/mdex_native/commit/8a240a287d9c494c5b390dffe1e73621a326bd22))

## [0.1.4](https://github.com/leandrocp/mdex_native/compare/v0.1.3...v0.1.4) (2026-06-06)


### Features

* add syntect syntax highlighter ([#12](https://github.com/leandrocp/mdex_native/issues/12)) ([57b7619](https://github.com/leandrocp/mdex_native/commit/57b76192eee2f44642c7c968495a2a0be02cfa87))

## [0.1.3](https://github.com/leandrocp/mdex_native/compare/v0.1.2...v0.1.3) (2026-06-06)


### Features

* explicit :syntax_highlight with :engine ([#11](https://github.com/leandrocp/mdex_native/issues/11)) ([b334d35](https://github.com/leandrocp/mdex_native/commit/b334d3529f9c34ecf166b34c0086be089e93305b))


### Documentation

* main page ([9ca4ae9](https://github.com/leandrocp/mdex_native/commit/9ca4ae9aa1f63c9e9ed1bdc1f2afc8e849cc9b2d))
* Update documentation for Comrak options mapping ([#10](https://github.com/leandrocp/mdex_native/issues/10)) ([0811ce8](https://github.com/leandrocp/mdex_native/commit/0811ce8c08c0ac04b5e43ceede76c27c11262af3)) by @josevalim

## [0.1.2](https://github.com/leandrocp/mdex_native/compare/v0.1.1...v0.1.2) (2026-06-05)


### Bug Fixes

* use From trait for Sourcepos ([#7](https://github.com/leandrocp/mdex_native/issues/7)) ([43a7424](https://github.com/leandrocp/mdex_native/commit/43a74243d7c1aa314f650d28d8adb2258d12b78e))

## [0.1.1](https://github.com/leandrocp/mdex_native/compare/v0.1.0...v0.1.1) (2026-06-05)


### Bug Fixes

* **deps:** optional rustler ([ca2b873](https://github.com/leandrocp/mdex_native/commit/ca2b873052bd8b66cd8c54bdb46f4fcf15460c80))

## 0.1.0 (2026-06-04)

### Features

- Initial release of `MDExNative`.
- Add Comrak Markdown parsing and rendering through `MDExNative.Comrak`.
- Add Ammonia HTML sanitization through `MDExNative.Ammonia`.
- Add Lumis syntax highlighting support through the native `LumisAdapter`.
