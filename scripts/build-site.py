#!/usr/bin/env python3
"""Assemble the static site (docs, WASM gallery, API docs) into one directory.

Run through scripts/build-site.sh, which builds the inputs and supplies the
`markdown` dependency. Fails when any internal link or heading anchor on a
generated page does not resolve.
"""
import argparse
import hashlib
import html
import json
import posixpath
import re
import shutil
import subprocess
from pathlib import Path
from urllib.parse import urlsplit

import markdown
from markdown.extensions import Extension
from markdown.extensions.toc import TocExtension
from markdown.treeprocessors import Treeprocessor

ROOT = Path(__file__).resolve().parent.parent
REPOSITORY = "https://github.com/mizorewww/gpui-kit-material"
GLUE = "gpui_kit_material_gallery_web.js"
WASM = "gpui_kit_material_gallery_web_bg.wasm"
# The icon styles the Gallery draws; crates/gallery-web/www/main.js lists the same.
SYMBOLS = ("MaterialSymbolsOutlined.ttf", "MaterialSymbolsRounded.ttf")

# Source path -> (published URL, navigation label). Optional docs are skipped
# until they exist; any other repository path links to GitHub.
PAGES = {
    "README.md": ("/", "Overview"),
    "docs/ARCHITECTURE.md": ("/docs/architecture/", "Architecture"),
    "docs/DEVELOPMENT.md": ("/docs/development/", "Development"),
    "docs/ROADMAP.md": ("/docs/roadmap/", "Roadmap"),
    "docs/PRODUCTION.md": ("/docs/production/", "Production"),
    "docs/COVERAGE.md": ("/docs/coverage/", "Coverage"),
}

API_CRATES = [
    ("gpui_kit_material", "gpui-kit-material",
     "Theme runtime, tokens, motion, icons and components."),
    ("gpui_kit_material_color", "gpui-kit-material-color",
     "Color roles, palettes, reference themes and scheme derivation."),
]

# Pages merges every matching rule, so Cache-Control rules must not overlap, and
# anything unlisted keeps its default `public, max-age=0, must-revalidate`.
HEADERS = """\
/*
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin

/
  Cache-Control: no-cache

/gallery/
  Cache-Control: no-cache
  Content-Security-Policy: default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; base-uri 'none'; form-action 'none'

/gallery/index.html
  Cache-Control: no-cache

/gallery/main.js
  Cache-Control: no-cache

/gallery/profile.js
  Cache-Control: no-cache

/gallery/wasm/*
  Cache-Control: public, max-age=31536000, immutable

/fonts/*
  Cache-Control: public, max-age=31536000, immutable

/api/static.files/*
  Cache-Control: public, max-age=31536000, immutable
"""

STYLE = """\
@font-face { font-family: Roboto; src: url("FONT_URL") format("woff2");
  font-weight: 100 900; font-display: swap; }
:root { color-scheme: light dark;
  --primary: #6750a4; --on-primary: #fff; --secondary-container: #e8def8;
  --on-secondary-container: #4a4459; --surface: #fef7ff; --on-surface: #1d1b20;
  --on-surface-variant: #49454f; --surface-container: #f3edf7;
  --surface-container-high: #ece6f0; --outline: #79747e; --outline-variant: #cac4d0; }
@media (prefers-color-scheme: dark) { :root {
  --primary: #d0bcff; --on-primary: #381e72; --secondary-container: #4a4458;
  --on-secondary-container: #e8def8; --surface: #141218; --on-surface: #e6e0e9;
  --on-surface-variant: #cac4d0; --surface-container: #211f26;
  --surface-container-high: #2b2930; --outline: #938f99; --outline-variant: #49454f; } }
* { box-sizing: border-box; }
html { scroll-padding-top: 80px; -webkit-text-size-adjust: 100%; }
body { margin: 0; background: var(--surface); color: var(--on-surface);
  font: 400 1rem/1.6 Roboto, system-ui, -apple-system, "Segoe UI", sans-serif; }
a { color: var(--primary); }
.bar { position: sticky; top: 0; z-index: 2; display: flex; align-items: center; gap: 4px;
  min-height: 64px; padding: 8px 16px; flex-wrap: wrap; background: var(--surface-container); }
.brand { margin-right: auto; padding: 0 8px; color: var(--on-surface); font-size: 1.375rem;
  line-height: 1.75rem; text-decoration: none; }
.bar nav { display: flex; gap: 4px; flex-wrap: wrap; }
.bar nav a { padding: 8px 12px; border-radius: 20px; color: var(--on-surface-variant);
  font-weight: 500; font-size: .875rem; line-height: 1.5rem; text-decoration: none; }
.bar nav a:hover { background: var(--surface-container-high); }
.layout { display: grid; grid-template-columns: 240px minmax(0, 1fr); gap: 32px;
  max-width: 1200px; margin: 0 auto; padding: 24px 16px 64px; }
.side { position: sticky; top: 88px; align-self: start; max-height: calc(100vh - 104px);
  overflow-y: auto; font-size: .875rem; }
.side summary { display: none; }
.side ul { list-style: none; margin: 0; padding: 0; }
.side a { display: block; padding: 8px 16px; border-radius: 28px; font-weight: 500;
  color: var(--on-surface-variant); text-decoration: none; }
.side a:hover { background: var(--surface-container); }
.side a[aria-current] { background: var(--secondary-container); color: var(--on-secondary-container); }
.side .toc { margin: 12px 0 0; padding-top: 12px; border-top: 1px solid var(--outline-variant); }
.side .toc a { padding: 4px 16px; font-weight: 400; }
main { min-width: 0; }
h1 { margin: 0 0 16px; font-size: 2.25rem; line-height: 2.75rem; font-weight: 400; }
h2 { margin: 40px 0 12px; font-size: 1.75rem; line-height: 2.25rem; font-weight: 400; }
h3 { margin: 28px 0 8px; font-size: 1.375rem; line-height: 1.75rem; font-weight: 500; }
h4 { margin: 20px 0 8px; font-size: 1rem; font-weight: 500; }
main p, main li { max-width: 75ch; }
code { padding: .1em .35em; border-radius: 6px; background: var(--surface-container);
  font: .875em/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; overflow-wrap: anywhere; }
pre { padding: 16px; border-radius: 12px; overflow-x: auto; background: var(--surface-container); }
pre code { padding: 0; background: none; overflow-wrap: normal; }
blockquote { margin: 16px 0; padding: 12px 16px; border-radius: 12px;
  background: var(--secondary-container); color: var(--on-secondary-container); }
blockquote p { margin: 0; }
.table { margin: 16px 0; overflow-x: auto; border: 1px solid var(--outline-variant); border-radius: 12px; }
table { width: 100%; border-collapse: collapse; font-size: .875rem; }
th, td { padding: 10px 16px; text-align: left; vertical-align: top;
  border-bottom: 1px solid var(--outline-variant); }
tr:last-child td { border-bottom: 0; }
th { font-weight: 500; background: var(--surface-container); }
.actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 24px 0; }
.actions a { padding: 10px 24px; border-radius: 20px; font-weight: 500; font-size: .875rem;
  line-height: 1.25rem; text-decoration: none; }
.actions .filled { background: var(--primary); color: var(--on-primary); }
.actions .tonal { background: var(--secondary-container); color: var(--on-secondary-container); }
.actions .outlined { border: 1px solid var(--outline); color: var(--primary); }
footer { padding: 16px; color: var(--on-surface-variant); font-size: .75rem; text-align: center; }
:focus-visible { outline: 3px solid var(--primary); outline-offset: 2px; }
@media (max-width: 840px) {
  .layout { grid-template-columns: minmax(0, 1fr); gap: 16px; }
  .side { position: static; max-height: none; }
  .side summary { display: block; padding: 8px 16px; border-radius: 12px; cursor: pointer;
    background: var(--surface-container); font-weight: 500; }
  h1 { font-size: 1.75rem; line-height: 2.25rem; }
  h2 { font-size: 1.5rem; line-height: 2rem; }
}
"""

TEMPLATE = """\
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta name="theme-color" media="(prefers-color-scheme: light)" content="#f3edf7">
<meta name="theme-color" media="(prefers-color-scheme: dark)" content="#211f26">
<title>{title}</title>
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Crect width='24' height='24' rx='7' fill='%236750a4'/%3E%3Ccircle cx='12' cy='12' r='5' fill='%23fff'/%3E%3C/svg%3E">
<link rel="stylesheet" href="{stylesheet}">
</head>
<body>
<header class="bar">
<a class="brand" href="/">gpui-kit-material</a>
<nav aria-label="Site">
<a href="/gallery/">Gallery</a>
<a href="/api/">API</a>
<a href="{repository}">GitHub</a>
</nav>
</header>
<div class="layout">
<div class="side">{side}</div>
<main>
{body}
</main>
</div>
<footer>MIT licensed. Built from <a href="{repository}/commit/{commit}">{short_commit}</a>.</footer>
</body>
</html>
"""


def github_slug(value, _separator):
    """GitHub's heading ids, so links written for GitHub keep working here."""
    value = re.sub(r"[^\w\- ]", "", value.strip().lower(), flags=re.UNICODE)
    return value.replace(" ", "-")


class SiteLinks(Treeprocessor):
    """Point repository-relative links at published pages or at GitHub, and
    wrap tables so that only the table scrolls on a narrow screen."""

    def __init__(self, md, source, published):
        super().__init__(md)
        self.source, self.published = source, published

    def rewrite(self, target):
        parts = urlsplit(target)
        if parts.scheme or parts.netloc or not parts.path:
            return target
        path = posixpath.normpath(posixpath.join(posixpath.dirname(self.source), parts.path))
        fragment = f"#{parts.fragment}" if parts.fragment else ""
        if path in self.published:
            return self.published[path] + fragment
        kind = "tree" if (ROOT / path).is_dir() else "blob"
        return f"{REPOSITORY}/{kind}/main/{path}{fragment}"

    def run(self, root):
        for element in root.iter():
            for attribute in ("href", "src"):
                if attribute in element.attrib:
                    element.set(attribute, self.rewrite(element.get(attribute)))
        for parent in list(root.iter()):
            for index, child in enumerate(list(parent)):
                if child.tag == "table":
                    wrapper = parent.makeelement("div", {"class": "table"})
                    parent.remove(child)
                    wrapper.append(child)
                    parent.insert(index, wrapper)


class SiteExtension(Extension):
    def __init__(self, source, published):
        super().__init__()
        self.source, self.published = source, published

    def extendMarkdown(self, md):
        md.treeprocessors.register(SiteLinks(md, self.source, self.published), "site_links", 1)


def content_hash(data):
    return hashlib.sha256(data).hexdigest()[:16]


def side_navigation(pages, current, toc=""):
    items = "".join(
        f'<li><a href="{url}"{" aria-current=page" if url == current else ""}>{html.escape(label)}</a></li>'
        for url, label in pages)
    label = next((label for url, label in pages if url == current), "Pages")
    # Open without JavaScript; collapsed on a narrow screen, where it precedes the text.
    return (f'<details id="side" open><summary>{html.escape(label)}</summary>'
            f'<nav aria-label="Documentation"><ul>{items}</ul></nav>{toc}</details>'
            "<script>if (matchMedia('(max-width: 840px)').matches) "
            "document.getElementById('side').open = false;</script>")


def table_of_contents(tokens):
    def walk(entries):
        return "".join(f'<li><a href="#{entry["id"]}">{entry["name"]}</a>'
                       f'{walk(entry["children"]) if entry["level"] < 2 else ""}</li>'
                       for entry in entries)

    # The single H1 is the page title, so list its H2 children.
    entries = tokens[0]["children"] if len(tokens) == 1 else tokens
    if not entries:
        return ""
    return f'<nav class="toc" aria-label="On this page"><ul>{walk(entries)}</ul></nav>'


def write_page(output, url, **values):
    path = output / url.lstrip("/") / "index.html"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(TEMPLATE.format(repository=REPOSITORY, **values))


def build_assets(output):
    """The stylesheet and font, named by content so they can be cached forever."""
    fonts = output / "fonts"
    fonts.mkdir(parents=True)
    source = ROOT / "crates/material/examples/gallery/fonts/roboto"
    font = (source / "Roboto-latin-wght.woff2").read_bytes()
    font_name = f"roboto-latin-wght.{content_hash(font)}.woff2"
    (fonts / font_name).write_bytes(font)
    shutil.copy(source / "OFL.txt", fonts / "Roboto-OFL.txt")
    style = STYLE.replace("FONT_URL", font_name).encode()
    style_name = f"site.{content_hash(style)}.css"
    (fonts / style_name).write_bytes(style)
    return f"/fonts/{style_name}"


def build_docs(output, stylesheet, commit):
    present = {source: page for source, page in PAGES.items() if (ROOT / source).exists()}
    published = {source: url for source, (url, _) in present.items()}
    navigation = [(url, label) for url, label in present.values()]
    common = dict(stylesheet=stylesheet, commit=commit, short_commit=commit[:7])
    for source, (url, label) in present.items():
        converter = markdown.Markdown(extensions=[
            "tables", "fenced_code", "sane_lists",
            TocExtension(slugify=github_slug, toc_depth="1-3"),
            SiteExtension(source, published),
        ])
        body = converter.convert((ROOT / source).read_text())
        if url == "/":
            actions = ('<div class="actions">'
                       '<a class="filled" href="/gallery/">Open the gallery</a>'
                       '<a class="tonal" href="/api/">API documentation</a>'
                       f'<a class="outlined" href="{REPOSITORY}">GitHub</a></div>')
            body = body.replace("</h1>", f"</h1>\n{actions}", 1)
        title = "gpui-kit-material" if url == "/" else f"{label} · gpui-kit-material"
        toc = table_of_contents(converter.toc_tokens)
        write_page(output, url, title=title, body=body,
                   side=side_navigation(navigation, url, toc), **common)

    rows = "".join(f'<tr><td><a href="/api/{module}/"><code>{name}</code></a></td><td>{purpose}</td></tr>'
                   for module, name, purpose in API_CRATES)
    write_page(output, "/api/", title="API · gpui-kit-material",
               side=side_navigation(navigation, "/api/"), **common,
               body=("<h1>API documentation</h1>"
                     '<div class="table"><table><thead><tr><th>Crate</th><th>Purpose</th></tr></thead>'
                     f"<tbody>{rows}</tbody></table></div>"))
    missing = TEMPLATE.format(
        repository=REPOSITORY, title="Not found · gpui-kit-material",
        side=side_navigation(navigation, "/404"), **common,
        body='<h1>Page not found</h1><p>Return to the <a href="/">overview</a>.</p>')
    (output / "404.html").write_text(missing)


def build_gallery(output):
    """Copy the gallery with content-hashed bindings; every path stays relative,
    so the page works under /gallery/ as well as at a local server's root."""
    www = ROOT / "crates/gallery-web/www"
    target = output / "gallery"
    (target / "wasm").mkdir(parents=True)
    for name in ("main.js", "profile.js"):
        shutil.copy(www / name, target / name)
    wasm = (www / "wasm" / WASM).read_bytes()
    glue = (www / "wasm" / GLUE).read_bytes()
    wasm_name = f"gallery.{content_hash(wasm)}.wasm"
    glue_name = f"gallery.{content_hash(glue)}.js"
    (target / "wasm" / wasm_name).write_bytes(wasm)
    (target / "wasm" / glue_name).write_bytes(glue)
    symbols, total = {}, len(wasm)
    for name in SYMBOLS:
        font = (www / "wasm" / name).read_bytes()
        hashed = f"{Path(name).stem}.{content_hash(font)}.ttf"
        (target / "wasm" / hashed).write_bytes(font)
        symbols[name] = f"./wasm/{hashed}"
        total += len(font)
    assets = json.dumps({"glue": f"./wasm/{glue_name}", "wasm": f"./wasm/{wasm_name}",
                         "symbols": symbols, "totalBytes": total})
    preloads = "".join(f'\n    <link rel="preload" as="fetch" crossorigin href="{path}">'
                       for path in symbols.values())
    page = (www / "index.html").read_text()
    for marker, value in (
        ("<!--gallery-preload-->",
         f'<link rel="modulepreload" href="./wasm/{glue_name}">\n'
         f'    <link rel="preload" as="fetch" crossorigin href="./wasm/{wasm_name}">{preloads}'),
        ("<!--gallery-assets-->",
         f'<script type="application/json" id="gallery-assets">{assets}</script>'),
    ):
        if marker not in page:
            raise SystemExit(f"{marker} is missing from the gallery page")
        page = page.replace(marker, value)
    (target / "index.html").write_text(page)


def check_links(output):
    """Every internal link and anchor on the generated pages must resolve."""
    def destination(path):
        path = output / path.lstrip("/")
        return path / "index.html" if path.is_dir() else path

    failures = []
    # rustdoc's own pages hold script templates and are not ours to check.
    pages = [page for page in output.rglob("*.html")
             if page.relative_to(output).parts[0] != "api" or page == output / "api/index.html"]
    for page in pages:
        text = page.read_text()
        for target in re.findall(r'(?:href|src)="([^"]+)"', text):
            target = html.unescape(target)
            parts = urlsplit(target)
            if parts.scheme or parts.netloc:
                continue
            resolved = (destination(parts.path) if parts.path.startswith("/")
                        else page.parent / parts.path) if parts.path else page
            if not resolved.is_file():
                failures.append(f"{page.relative_to(output)}: {target}")
            elif parts.fragment and resolved.suffix == ".html":
                ids = re.findall(r'\bid="([^"]+)"', resolved.read_text())
                if parts.fragment not in ids:
                    failures.append(f"{page.relative_to(output)}: {target} (no such anchor)")
    if failures:
        raise SystemExit("Broken links:\n  " + "\n  ".join(failures))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--api-docs", type=Path, required=True, help="cargo doc output directory")
    args = parser.parse_args()
    output = args.output
    shutil.rmtree(output, ignore_errors=True)
    output.mkdir(parents=True)
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, check=True,
                            capture_output=True, text=True).stdout.strip()
    shutil.copytree(args.api_docs, output / "api",
                    ignore=shutil.ignore_patterns(".lock", ".rustc_info.json", "*.lock"))
    build_gallery(output)
    build_docs(output, build_assets(output), commit)
    (output / "_headers").write_text(HEADERS)
    check_links(output)
    files = [path for path in output.rglob("*") if path.is_file()]
    print(f"{output}: {len(files)} files, {sum(path.stat().st_size for path in files) / 1e6:.1f} MB")


if __name__ == "__main__":
    main()
