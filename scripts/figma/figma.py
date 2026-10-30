#!/usr/bin/env python3
"""Read Figma Material 3 Design Kit values through the Plugin API.

Runs read-only queries in a Figma editor tab through the Playwright CLI and
regenerates the token fixtures. See scripts/README.md for setup.

    figma.py attach
    figma.py --file-key KEY tokens [--raw-dir DIR] [--check]
    figma.py --file-key KEY tokens --from-raw DIR [--check]
    figma.py --file-key KEY inspect "Buttons" "Button - tonal" [--depth 6]
    figma.py --file-key KEY export "Buttons" /tmp/buttons [--max-dim 2400]

The file key comes from --file-key or FIGMA_FILE_KEY. Every live command checks
it against the open tab, so a query never reads the wrong file. Nothing here
writes to Figma, and no credential is read from or written to a file: `attach`
passes PLAYWRIGHT_MCP_EXTENSION_TOKEN to the CLI only through its environment.
"""
import argparse
import base64
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
QUERIES = Path(__file__).resolve().parent / 'queries'
PLAYWRIGHT = ['npx', '-y', '@playwright/cli@latest']
TOKEN_ENV = 'PLAYWRIGHT_MCP_EXTENSION_TOKEN'
QUERY_TIMEOUT_S = 300

FIXTURES = {
    'colors': ROOT / 'crates/color/tests/fixtures/figma-baseline-colors.json',
    'typography': ROOT / 'crates/material/tests/fixtures/figma-typography.json',
    'sys': ROOT / 'crates/material/tests/fixtures/figma-sys-tokens.json',
    'button': ROOT / 'crates/material/tests/fixtures/figma-button.json',
}
RAW_QUERIES = {
    'figma-ids.json': 'ids.js',
    'figma-styles.json': 'styles.js',
    'figma-m3-colors.json': 'm3-colors.js',
    'figma-button.json': 'button.js',
}
REFERENCE = 'Google Material 3 Design Kit (Figma Community)'
MODES = [
    ('light', 'standard', 'Light'),
    ('light', 'medium', 'Light Medium Contrast'),
    ('light', 'high', 'Light High Contrast'),
    ('dark', 'standard', 'Dark'),
    ('dark', 'medium', 'Dark Medium Contrast'),
    ('dark', 'high', 'Dark High Contrast'),
]
TYPE_ROLES = [
    'display-large', 'display-medium', 'display-small',
    'headline-large', 'headline-medium', 'headline-small',
    'title-large', 'title-medium', 'title-small',
    'body-large', 'body-medium', 'body-small',
    'label-large', 'label-medium', 'label-small',
]
FONT_WEIGHTS = {'Regular': 400, 'Medium': 500, 'SemiBold': 600, 'Bold': 700}


class Session:
    """One named Playwright CLI session attached to a Figma editor tab."""

    def __init__(self, name, file_key=None):
        self.name = name
        self.file_key = file_key
        self.checked = False

    def run(self, *args):
        result = subprocess.run(
            [*PLAYWRIGHT, f'-s={self.name}', *args],
            capture_output=True, text=True, timeout=QUERY_TIMEOUT_S,
        )
        return result.stdout + result.stderr

    def query(self, script, **params):
        if not self.checked:
            self.checked = True
            self.check_file()
        return self.evaluate(script, params)

    def evaluate(self, script, params):
        source = (QUERIES / script).read_text().strip()
        wrapped = f'async () => await ({source})({json.dumps(params)})'
        output = self.run('eval', wrapped)
        match = re.search(r'### Result\n(.*)', output)
        if not match:
            raise SystemExit(f'{script}: no result from the Figma tab\n{output[-2000:]}')
        return json.loads(match.group(1))

    def check_file(self):
        if not self.file_key:
            raise SystemExit('Pass --file-key or set FIGMA_FILE_KEY')
        output = self.run('eval', 'async () => figma.fileKey')
        match = re.search(r'### Result\n(.*)', output)
        if not match:
            raise SystemExit(
                f'No Figma tab in session "{self.name}". Run `figma.py attach`, then open '
                f'https://www.figma.com/design/{self.file_key}/ in that tab.\n{output[-2000:]}')
        open_key = json.loads(match.group(1))
        if open_key != self.file_key:
            raise SystemExit(f'The open tab is file {open_key}, not {self.file_key}')


def write_json(path, value, check):
    """Write `value` the way the fixtures are formatted, or compare with it."""
    text = json.dumps(value, indent=2) + '\n'
    if check:
        matches = path.exists() and path.read_text() == text
        print(f'{"unchanged" if matches else "DIFFERS"}: {path.relative_to(ROOT)}')
        return matches
    path.write_text(text)
    print(f'wrote {path.relative_to(ROOT)}')
    return True


def figma_file(ids, file_key):
    return f'{ids["root"]}, file key {file_key}'


def collection(ids, name):
    return next(c for c in ids['colls'] if c['name'] == name)


def button_set(ids, name):
    return next(s['id'] for s in ids['buttonSets'] if s['name'] == name)


def colors_fixture(ids, rows, file_key):
    m3 = collection(ids, 'M3')
    mode_ids = {m['name']: m['modeId'] for m in m3['modes']}
    schemes = {
        r['name'][len('Schemes/'):].lower().replace(' ', '-'): r['vals']
        for r in rows if r['name'].startswith('Schemes/')
    }
    return {
        'reference': REFERENCE,
        'figma_file': figma_file(ids, file_key),
        'collection': f'M3 ({m3["id"]}), variables Schemes/*',
        'extraction': 'Figma Plugin API figma.variables.getLocalVariableCollectionsAsync(); aliases resolved per mode; sRGB rounded to 8-bit hex',
        'scope': 'The kit defines only the baseline (purple) scheme in six modes; its Monochrome/Pink/... modes are not md3.io themes. background, on-background and surface-tint are recorded but have no crate ColorRole (the crate paints surface, on-surface and primary).',
        'rows': [
            {
                'theme': 'baseline', 'mode': mode, 'contrast': contrast,
                'figma_mode': name, 'figma_mode_id': mode_ids[name],
                'colors': {role: vals[name].lower() for role, vals in sorted(schemes.items())},
            }
            for mode, contrast, name in MODES
        ],
    }


def typography_fixture(ids, styles_doc, file_key):
    typescale = {r['name']: r['vals']['Baseline'] for r in styles_doc['vars']['Typescale']}
    fonts = {r['name']: r['vals'] for r in styles_doc['vars']['Font theme']}
    styles = {s['name']: s for s in styles_doc['textStyles']}

    def row(name, emphasized):
        group, size = name.split('-')
        style_name = f'M3/{group}/{size}' + ('-emphasized' if emphasized else '')
        s = styles[style_name]
        if s['lh']['unit'] != 'PIXELS' or s['ls']['unit'] != 'PIXELS':
            raise SystemExit(f'{style_name}: expected pixel line height and letter spacing')
        return {
            'name': name,
            'figma_style': style_name,
            'figma_style_key': s['key'],
            'typeface': typescale[s['bound']['fontFamily']].split('/')[-1].lower(),
            'font_family': s['family'],
            'font_style': s['style'],
            'font_weight': FONT_WEIGHTS[s['style']],
            'font_size': s['size'],
            'line_height': s['lh']['value'],
            'letter_spacing': round(s['ls']['value'], 4),
        }

    return {
        'reference': REFERENCE,
        'figma_file': figma_file(ids, file_key),
        'figma_page': f'Styles ({ids["stylesPage"]})',
        'source': f'Local text styles M3/<group>/<size>[-emphasized], each bound to the Typescale collection ({collection(ids, "Typescale")["id"]}, mode Baseline) and the Font theme collection ({collection(ids, "Font theme")["id"]}, mode Baseline)',
        'extraction': 'Figma Plugin API figma.getLocalTextStylesAsync(); weights map font style names Regular/Medium/SemiBold to 400/500/600; letter spacing rounded from float32 to 4 decimals',
        'unit': 'design px at 100% text scale',
        'font_theme': {
            'brand': fonts['Static/Font/Brand']['Baseline'],
            'plain': fonts['Static/Font/Plain']['Baseline'],
            'weight_regular': fonts['Static/Weight/Regular']['Baseline'],
            'weight_medium': fonts['Static/Weight/Medium']['Baseline'],
            'weight_bold': fonts['Static/Weight/Bold']['Baseline'],
        },
        'roles': [row(n, False) for n in TYPE_ROLES],
        'emphasized': [row(n, True) for n in TYPE_ROLES],
    }


def sys_fixture(ids, styles_doc, color_rows, file_key):
    shape = {r['name'].split('/')[1].lower(): r['vals']['Baseline'] for r in styles_doc['vars']['Shape']}
    elevation = {}
    for s in styles_doc['effectStyles']:
        m = re.match(r'M3/Elevation (Light|Dark)/(\d)', s['name'])
        layers = []
        for e in s['effects']:
            color, alpha = e['c'].split('@')
            layers.append({'x': e['x'], 'y': e['y'], 'blur': e['r'], 'spread': e['s'], 'color': color, 'opacity': round(float(alpha), 3)})
        elevation.setdefault(m.group(1).lower(), {})[m.group(2)] = {
            'figma_style': s['name'], 'figma_style_id': s['id'], 'layers': layers,
        }
    opacities = sorted({
        round(float(r['vals']['Light'].split('@')[1]), 3)
        for r in color_rows if r['name'].startswith('State Layers/')
    })
    button = button_set(ids, 'Button')
    return {
        'reference': REFERENCE,
        'figma_file': figma_file(ids, file_key),
        'extraction': 'Figma Plugin API: figma.variables (Shape and M3 collections), figma.getLocalEffectStylesAsync(), and the Buttons page component sets',
        'shape_corner_px': {
            'source': f'Shape variable collection ({collection(ids, "Shape")["id"]}, mode Baseline), variables Corner/*',
            **shape,
        },
        'state_layer_opacity': {
            'source': f'M3 variable collection ({collection(ids, "M3")["id"]}), State Layers/<role>/Opacity-08|10|16: every role defines exactly these three opacities. Buttons page (Button {button}, Type=Round, Size=Medium): State=Hovered paints State-layer at Opacity-08, State=Focused at Opacity-10, State=Pressed a Ripple at Opacity-10 over the hover layer; dragged uses the remaining Opacity-16.',
            'defined_opacities': opacities,
            'hover': 0.08, 'focus': 0.10, 'pressed': 0.10, 'dragged': 0.16,
        },
        'disabled': {
            'source': f'Buttons page, every Button*/Toggle button* COMPONENT_SET (e.g. Button {button}), State=Disabled: content on-surface under a 0.38 layer opacity, container State Layers/On Surface/Opacity-10',
            'content_opacity': 0.38,
            'button_container_opacity': 0.10,
        },
        'elevation': {
            'source': 'Local effect styles M3/Elevation Light/1-5 and M3/Elevation Dark/1-5; both themes share geometry and opacities and differ only in layer order',
            **elevation,
        },
    }


def button_fixture(ids, raw, file_key):
    return {
        'reference': REFERENCE,
        'figma_file': figma_file(ids, file_key),
        'figma_page': 'Buttons',
        'extraction': f'Figma Plugin API over the Buttons page component sets: sizes from Button ({button_set(ids, "Button")}) Type=Round State=Enabled (Content frame height, State-layer auto-layout padding [top, right, bottom, left] and item spacing, Icon width, label text style), Type=Square / State=Pressed Content corner radius, and Button - outline ({button_set(ids, "Button - outline")}) stroke weight; disabled colors from each set at Type=Round, Size=Medium, State=Disabled (container fill variable, content variable times State-layer and node opacity, Content stroke)',
        'unit': 'design px',
        'sizes': raw['sizes'],
        'disabled': raw['disabled'],
    }


def cmd_attach(args):
    if not os.environ.get(TOKEN_ENV):
        raise SystemExit(
            f'Set {TOKEN_ENV} to the token the Playwright browser extension shows; '
            'it is read only from the environment.')
    print(Session(args.session).run('attach', f'--extension={args.browser}'))


def cmd_tokens(args):
    raw_dir = Path(args.from_raw or args.raw_dir)
    if not args.from_raw:
        raw_dir.mkdir(parents=True, exist_ok=True)
        session = Session(args.session, args.file_key)
        for name, script in RAW_QUERIES.items():
            (raw_dir / name).write_text(session.query(script) + '\n')
            print(f'queried {script}')
    raw = {name: json.loads((raw_dir / name).read_text()) for name in RAW_QUERIES}
    ids, styles, colors = raw['figma-ids.json'], raw['figma-styles.json'], raw['figma-m3-colors.json']
    # Offline runs cite the file key the raw dump was taken from.
    file_key = args.file_key or ids['fileKey']
    if file_key != ids['fileKey']:
        raise SystemExit(f'The raw dump is file {ids["fileKey"]}, not {file_key}')
    documents = {
        'colors': colors_fixture(ids, colors, file_key),
        'typography': typography_fixture(ids, styles, file_key),
        'sys': sys_fixture(ids, styles, colors, file_key),
        'button': button_fixture(ids, raw['figma-button.json'], file_key),
    }
    results = [write_json(FIXTURES[key], value, args.check) for key, value in documents.items()]
    if not all(results):
        raise SystemExit(1)


def cmd_inspect(args):
    session = Session(args.session, args.file_key)
    print(session.query('inspect.js', page=args.page, query=args.query, depth=args.depth))


def cmd_export(args):
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    session = Session(args.session, args.file_key)
    for index in range(session.query('page-size.js', page=args.page)):
        result = session.query('export-node.js', page=args.page, index=index, maxDim=args.max_dim)
        name, size, scale, png = result.split('|', 3)
        slug = re.sub(r'[^A-Za-z0-9]+', '-', name).strip('-') or 'section'
        (out / f'{index:02d}-{slug}.png').write_bytes(base64.b64decode(png))
        print(index, name, size, 'scale', scale)
    (out / 'outline.txt').write_text(session.query('outline.js', page=args.page) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--file-key', default=os.environ.get('FIGMA_FILE_KEY'))
    parser.add_argument('--session', default=os.environ.get('FIGMA_PW_SESSION', 'figma'))
    commands = parser.add_subparsers(dest='command', required=True)

    attach = commands.add_parser('attach', help='attach the session to the browser extension')
    attach.add_argument('--browser', default='chrome')
    attach.set_defaults(run=cmd_attach)

    tokens = commands.add_parser('tokens', help='regenerate the figma-*.json token fixtures')
    tokens.add_argument('--raw-dir', default='/tmp/figma-raw', help='where query results are kept')
    tokens.add_argument('--from-raw', help='rebuild from saved query results, without a browser')
    tokens.add_argument('--check', action='store_true', help='compare with the committed fixtures; write nothing')
    tokens.set_defaults(run=cmd_tokens)

    inspect = commands.add_parser('inspect', help='print the spec of one node')
    inspect.add_argument('page')
    inspect.add_argument('query', help='node name substring')
    inspect.add_argument('--depth', type=int, default=6)
    inspect.set_defaults(run=cmd_inspect)

    export = commands.add_parser('export', help='export a page as PNGs and an outline')
    export.add_argument('page')
    export.add_argument('out')
    export.add_argument('--max-dim', type=int, default=2400)
    export.set_defaults(run=cmd_export)

    args = parser.parse_args()
    args.run(args)


if __name__ == '__main__':
    sys.exit(main())
