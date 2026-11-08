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


class GalleryTargets(unittest.TestCase):
    MS = 1_000_000

    def frame(self, start, ms, step, initial=False, presented=True):
        return {"initial": initial, "step": step, "start_ns": start, "frame_ns": ms * self.MS,
                "present_ns": 1 if presented else None}

    def test_a_switch_frame_is_its_own_cost_not_steady_state(self):
        data = {"marks_ns": [["switch:button", 100]], "page_switch_ns": [9 * self.MS], "frames": [
            self.frame(-5, 4, "startup", initial=True, presented=False),
            self.frame(10, 0.5, "startup"),
            self.frame(120, 9, "switch:button"),
            self.frame(140, 0.8, "switch:button"),
        ]}
        table = profile.gallery_targets(data)
        self.assertEqual(table["metrics"]["steady_frame"]["max_ns"], 0.8 * self.MS)
        self.assertEqual(table["targets"]["steady_cpu_max"]["verdict"], "PASS")
        self.assertEqual(table["targets"]["page_switch"]["verdict"], "FAIL")
        # The initial draw and the first submitted frame after it.
        self.assertEqual(table["metrics"]["first_frame_ns"], 4.5 * self.MS)
        self.assertEqual(table["targets"]["first_frame_vs_kit"]["verdict"], "UNVERIFIED")

    def test_no_frames_judge_nothing(self):
        self.assertEqual(profile.gallery_targets({"frames": []})["targets"], {})


if __name__ == "__main__":
    unittest.main()
