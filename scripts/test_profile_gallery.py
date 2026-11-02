#!/usr/bin/env python3
"""Rules of profile-gallery.py that need no build or browser: python3 scripts/test_profile_gallery.py"""
import importlib.util
from pathlib import Path
import unittest

_SPEC = importlib.util.spec_from_file_location(
    "profile_gallery", Path(__file__).resolve().parent / "profile-gallery.py")
profile = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(profile)


class ProbeMovement(unittest.TestCase):
    URL = "https://md3.io/"

    def test_a_page_that_moved_needs_no_warning(self):
        self.assertIsNone(profile.probe_movement_warning(self.URL, True))

    def test_a_page_that_stood_still_measured_idle(self):
        self.assertIn("idle page", profile.probe_movement_warning(self.URL, False))

    def test_an_unknown_movement_is_reported_as_unknown_not_idle(self):
        warning = profile.probe_movement_warning(self.URL, None)
        self.assertIn("unknown", warning)
        self.assertNotIn("idle", warning)


if __name__ == "__main__":
    unittest.main()
