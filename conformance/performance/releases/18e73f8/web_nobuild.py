#!/usr/bin/env python3
"""Run scripts/profile-gallery.py unchanged, but skip the wasm rebuild step:
a release build + wasm-opt right before a run raises the load that the run is
judged under. The module was built by build-all.sh and placed in www/wasm."""
import importlib.util, subprocess, sys
PG = "/Users/aac6fef/Developer/gpui-kit-material/scripts/profile-gallery.py"
real = subprocess.run
def run(cmd, *a, **k):
    if isinstance(cmd, list) and cmd and cmd[0] == "./scripts/build-gallery-web.sh":
        return subprocess.CompletedProcess(cmd, 0)
    return real(cmd, *a, **k)
subprocess.run = run
# www/profile.js at this commit throws a ReferenceError in finish(): `targets` is read at line 148,
# before its `const` at line 170, so no report is ever produced. Serve a scratch copy with that
# one line moved below the declaration; every other file comes from the tree.
import http.server
_tp = http.server.SimpleHTTPRequestHandler.translate_path
def translate(self, path):
    if path.split("?")[0] == "/profile.js":
        return "%s/patched/profile.js" % __import__("os").path.dirname(__file__)
    return _tp(self, path)
http.server.SimpleHTTPRequestHandler.translate_path = translate
spec = importlib.util.spec_from_file_location("profile_gallery", PG)
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
sys.argv = [PG] + sys.argv[1:]
sys.exit(m.main())
