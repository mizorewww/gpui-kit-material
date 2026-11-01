#!/usr/bin/env python3
"""Run scripts/profile-gallery.py --bench unchanged except for one feature on the build:
engine_bench at ff0b2f2 does not compile with its shipped features (Window::layout_stats is
cfg(any(test, feature = "test-support")) in vendor/gpui-fast; the crate split removed the
dev-dependency that used to unify it). Adds gpui-kit/test-support from the command line."""
import importlib.util, subprocess, sys
PG = "/Users/aac6fef/Developer/gpui-kit-material/scripts/profile-gallery.py"
real = subprocess.run
def run(cmd, *a, **k):
    if isinstance(cmd, list) and "engine_bench" in cmd and "--features" in cmd and cmd[0] == "cargo":
        i = cmd.index("--features"); cmd = list(cmd); cmd[i + 1] += ",gpui-kit/test-support"
    return real(cmd, *a, **k)
subprocess.run = run
spec = importlib.util.spec_from_file_location("profile_gallery", PG)
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
sys.argv = [PG] + sys.argv[1:]
sys.exit(m.main())
