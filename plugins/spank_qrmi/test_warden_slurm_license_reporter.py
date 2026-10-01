"""Tests for the Warden to Slurm license reporter."""

import argparse
import importlib.util
from io import BytesIO
from pathlib import Path
from unittest.mock import patch


MODULE_PATH = Path(__file__).with_name("warden_slurm_license_reporter.py")
SPEC = importlib.util.spec_from_file_location(
    "warden_slurm_license_reporter", MODULE_PATH
)
reporter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(reporter)


def reporter_args() -> argparse.Namespace:
    """Build nominal reporter arguments."""
    return argparse.Namespace(
        warden_url="http://warden/accessible",
        warden_slots_url="http://warden/qpu-slots",
        resource="qpu_slots@warden",
        total_slots=10,
        timeout_seconds=3,
        interval_seconds=5,
        sacctmgr="sacctmgr",
        once=True,
    )


def test_report_once_updates_changed_usage():
    """Changed Warden usage updates Slurm."""
    args = reporter_args()
    with (
        patch.object(reporter, "read_warden_usage", return_value=(10, 4)),
        patch.object(reporter, "update_last_consumed") as update,
    ):
        assert reporter.report_once(args, None) == 4
    update.assert_called_once_with("sacctmgr", "qpu_slots@warden", 4)


def test_report_once_skips_unchanged_usage():
    """Unchanged usage avoids a Slurm database write."""
    args = reporter_args()
    with (
        patch.object(reporter, "read_warden_usage", return_value=(10, 4)),
        patch.object(reporter, "update_last_consumed") as update,
    ):
        assert reporter.report_once(args, 4) == 4
    update.assert_not_called()


def test_report_once_fails_closed():
    """Warden errors report the full slot count as consumed."""
    args = reporter_args()
    with (
        patch.object(reporter, "read_warden_usage", side_effect=TimeoutError),
        patch.object(reporter, "update_last_consumed") as update,
    ):
        assert reporter.report_once(args, None) == 10
    update.assert_called_once_with("sacctmgr", "qpu_slots@warden", 10)


def test_read_warden_usage_blocks_slots_during_maintenance():
    """Warden maintenance consumes the full Slurm license pool."""

    class Response(BytesIO):
        status = 200

        def __enter__(self):
            return self

        def __exit__(self, *_args):
            self.close()

    responses = [
        Response(b'{"is_accessible":false}'),
        Response(b'{"qpu_slots_total":10,"qpu_slots_used":0}'),
    ]
    with patch.object(reporter.urllib.request, "urlopen", side_effect=responses):
        assert reporter.read_warden_usage(
            "http://warden/accessible", "http://warden/qpu-slots", 3
        ) == (10, 10)
