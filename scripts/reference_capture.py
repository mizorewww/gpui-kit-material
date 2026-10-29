"""Shared plumbing for the pinned-reference capture drivers in this directory.

Each ``capture-*.py`` driver verifies the pinned reference checkout, bundles the
original reference modules with bun, compiles the reference Sass, serves the
result from a temporary directory and drives an ``agent-browser`` session. This
module owns those steps so that every driver applies the same pins, subprocess
timeouts and cleanup.
"""
import argparse
import contextlib
import functools
import http.server
import json
from pathlib import Path
import subprocess
import threading
import uuid

# Pinned reference checkout; every capture refuses any other HEAD.
REVISION = "c910072f3fc5b660276731d8a935d7186f288364"
SASS = "sass@1.85.1"
# Upper bound for any single external command. Whole-matrix sampler
# evaluations can take minutes; a hung tool must still terminate the capture.
COMMAND_TIMEOUT = 900


class CommandError(subprocess.CalledProcessError):
    """A failed command whose message includes its captured output."""

    def __str__(self):
        detail = (self.stderr or "") + (self.output or "")
        return f"{super().__str__()} {detail}".rstrip()


def run(*args, timeout=COMMAND_TIMEOUT, **kwargs):
    """Run a command, returning stdout and raising on failure or timeout."""
    result = subprocess.run(args, text=True, capture_output=True, timeout=timeout, **kwargs)
    if result.returncode:
        raise CommandError(result.returncode, args, result.stdout, result.stderr)
    return result.stdout


def parser(doc):
    """Argument parser for the ``source output`` capture interface."""
    result = argparse.ArgumentParser(description=doc)
    result.add_argument("source", type=Path)
    result.add_argument("output", type=Path)
    return result


def require_reference(parser, source, pathspec, head_message, clean_message):
    """Reject a reference checkout that is not the clean pinned revision."""
    if run("git", "-C", str(source), "rev-parse", "HEAD").strip() != REVISION:
        parser.error(head_message + REVISION)
    if run("git", "-C", str(source), "status", "--porcelain", "--", pathspec):
        parser.error(clean_message)


def tool_versions():
    """Versions of the bundler, Sass compiler and browser driver."""
    return {
        "bun": run("bun", "--version").strip(),
        "sass": run("npx", "--yes", SASS, "--version").strip(),
        "agent_browser": run("agent-browser", "--version").strip(),
    }


def module(path):
    """A JavaScript string literal naming a reference module."""
    return json.dumps(str(path))


def bundle(work, entry):
    """Bundle ``entry`` TypeScript into ``work/entry.js``."""
    (work / "entry.ts").write_text(entry)
    run("bun", "build", str(work / "entry.ts"), "--outfile", str(work / "entry.js"))


def compile_sass(stylesheet, output):
    """Compile ``stylesheet`` with the pinned Sass release."""
    run("npx", "--yes", SASS, "--no-source-map", "--style=expanded", str(stylesheet), str(output))


def sampler(script):
    """The browser sampler that accompanies a driver (``capture-x.js``)."""
    return Path(script).with_suffix(".js").read_text()


@contextlib.contextmanager
def serve(directory):
    """Serve ``directory`` on an ephemeral loopback port, yielding the port."""
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield server.server_port
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


class Browser:
    """One named ``agent-browser`` session."""

    def __init__(self, prefix):
        self.session = prefix + uuid.uuid4().hex[:12]

    def __call__(self, *commands, **kwargs):
        return run("agent-browser", "--session", self.session, *commands, **kwargs)

    def eval_text(self, code):
        return self("eval", "--stdin", input=code)

    def evaluate(self, code):
        return json.loads(self.eval_text(code))

    def close(self):
        self("close")


@contextlib.contextmanager
def browser_session(prefix):
    """Yield a ``Browser`` that is always closed, without masking a failure."""
    browser = Browser(prefix)
    try:
        yield browser
    except BaseException:
        with contextlib.suppress(Exception):
            browser.close()
        raise
    browser.close()


def write_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")
