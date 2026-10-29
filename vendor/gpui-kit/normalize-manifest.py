#!/usr/bin/env python3
"""Reproduce the local manifest from the pinned, unmodified Kit checkout."""
import hashlib
import difflib
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib

REVISION = 'bb85882e1faab845d9e7be33b69a152879a90f69'
REPOSITORY = 'https://github.com/longbridge/gpui-kit'
source = Path(sys.argv[1]).resolve()
destination = Path(__file__).resolve().parent
actual = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
if actual != REVISION:
    raise SystemExit(f'Expected {REVISION}, got {actual}')
if subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain', '--', 'Cargo.toml', 'crates/kit'], text=True):
    raise SystemExit('Source manifests and crates/kit must be unmodified')
workspace = tomllib.loads((source / 'Cargo.toml').read_text())['workspace']
original = (source / 'crates/kit/Cargo.toml').read_text()
if (destination / 'Cargo.toml.orig').read_text() != original:
    raise SystemExit('Cargo.toml.orig does not match the pinned source')

def value(item):
    if isinstance(item, bool):
        return 'true' if item else 'false'
    if isinstance(item, str):
        return json.dumps(item)
    if isinstance(item, list):
        return '[' + ', '.join(value(child) for child in item) + ']'
    if isinstance(item, dict):
        return '{ ' + ', '.join(f'{key} = {value(child)}' for key, child in item.items()) + ' }'
    return str(item)

def dependency(name, local=None):
    declared = workspace['dependencies'][name]
    result = dict(declared) if isinstance(declared, dict) else {'version': declared}
    if 'path' in result:
        del result['path']
        result.update(git=REPOSITORY, branch='main')
    local = dict(local or {})
    local.pop('workspace', None)
    if 'features' in local:
        local['features'] = list(dict.fromkeys(result.get('features', []) + local['features']))
    result.update(local)
    return result

manifest = original.replace('edition.workspace = true', f'edition = {value(workspace["package"]["edition"])}')
lints = '\n'.join(f'{name} = {value(setting)}' for name, setting in workspace['lints']['clippy'].items())
manifest = manifest.replace('[lints]\nworkspace = true', '[lints.clippy]\n' + lints)
manifest = re.sub(r'^([\w-]+)\.workspace = true$', lambda match: f'{match[1]} = {value(dependency(match[1]))}', manifest, flags=re.M)
manifest = re.sub(r'^([\w-]+) = (\{ workspace = true[^\n]*\})$',
    lambda match: f'{match[1]} = {value(dependency(match[1], tomllib.loads("entry = " + match[2])["entry"]))}', manifest, flags=re.M)
manifest = manifest.replace(
    '[target.\'cfg(not(any(target_os = "ios", target_os = "android")))\'.dependencies]',
    '[target.\'cfg(not(any(target_os = "ios", target_os = "android", target_family = "wasm")))\'.dependencies]')
wasm = '[target.\'cfg(target_family = "wasm")\'.dependencies]\n'
for name in ['gpui_platform', 'gpui_web', 'gpui-fast-platform', 'gpui-fast-web']:
    options = {'default-features': False}
    if name.startswith('gpui-fast-'):
        options['optional'] = True
    wasm += f'{name} = {value(dependency(name, options))}\n'
start = manifest.index('[target.\'cfg(target_family = "wasm")\'.dependencies]')
end = manifest.index('[dev-dependencies]', start)
manifest = manifest[:start] + wasm + '\n' + manifest[end:]
manifest = manifest.replace('default = ["component", "assets"]', '''default = ["component", "assets"]
# Explicit opt-in; default WASM builds use the single-threaded platform.
# This restores upstream worker/atomics support and its toolchain/runtime requirements.
multithreaded-web = ["gpui_platform/multithreaded", "gpui_web/multithreaded", "gpui-fast-platform?/multithreaded", "gpui-fast-web?/multithreaded"]''', 1)
# Parse the result before writing: no workspace inheritance may remain.
parsed = tomllib.loads(manifest)
assert parsed['package']['edition'] == '2024'
assert 'workspace = true' not in manifest and '.workspace' not in manifest
(destination / 'Cargo.toml').write_text(manifest)
(destination / 'MANIFEST.diff').write_text(''.join(difflib.unified_diff(original.splitlines(keepends=True), manifest.splitlines(keepends=True), fromfile='Cargo.toml.orig', tofile='Cargo.toml')))
files = {}
for path in sorted((source / 'crates/kit').rglob('*')):
    if path.is_file():
        files[str(path.relative_to(source / 'crates/kit'))] = hashlib.sha256(path.read_bytes()).hexdigest()
metadata = {
    'repository': REPOSITORY,
    'revision': REVISION,
    'subdirectory': 'crates/kit',
    'source_workspace_manifest_sha256': hashlib.sha256((source / 'Cargo.toml').read_bytes()).hexdigest(),
    'source_files_sha256': files,
    'source_licenses_sha256': {name: hashlib.sha256((source / name).read_bytes()).hexdigest() for name in ['LICENSE-APACHE', 'LICENSE-DOCS.md']},
    'normalized_manifest_sha256': hashlib.sha256(manifest.encode()).hexdigest(),
}
(destination / 'SOURCE.json').write_text(json.dumps(metadata, indent=2) + '\n')
