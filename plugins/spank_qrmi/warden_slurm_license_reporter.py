#!/usr/bin/env python3
"""Report Warden QPU usage to a Slurm remote license."""

import argparse
import json
import subprocess
import sys
import time
import urllib.request
from collections.abc import Sequence


def read_warden_usage(url: str, timeout_seconds: float) -> tuple[int, int]:
    """Read validated QPU slot totals and usage from Warden.

    Args:
        url (str): Warden accessible endpoint URL.
        timeout_seconds (float): HTTP request timeout in seconds.

    Returns:
        total (int): Configured total QPU slots.
        used (int): Currently used QPU slots.
    """
    with urllib.request.urlopen(url, timeout=timeout_seconds) as response:
        if response.status < 200 or response.status >= 300:
            raise RuntimeError(f"Warden returned HTTP {response.status}")
        payload = json.load(response)
    total = payload.get("qpu_slots_total")
    used = payload.get("qpu_slots_used")
    accessible = payload.get("is_accessible")
    if (
        not isinstance(accessible, bool)
        or not isinstance(total, int)
        or not isinstance(used, int)
        or total < 1
        or used < 0
        or used > total
    ):
        raise ValueError("Warden returned invalid QPU slot state")
    return total, used if accessible else total


def update_last_consumed(command: str, resource: str, used: int) -> None:
    """Update Slurm's externally consumed count.

    Args:
        command (str): sacctmgr executable path.
        resource (str): Slurm remote-license resource name.
        used (int): Externally reported consumed count.
    """
    subprocess.run(
        [
            command,
            "-i",
            "update",
            "resource",
            resource,
            "set",
            f"lastconsumed={used}",
        ],
        check=True,
    )


def report_once(args: argparse.Namespace, previous: int | None) -> int:
    """Poll Warden and update Slurm when the value changed.

    Args:
        args (argparse.Namespace): Parsed command-line arguments.
        previous (int | None): Last successfully reported value.

    Returns:
        (int): Reported consumed count.
    """
    try:
        total, used = read_warden_usage(args.warden_url, args.timeout_seconds)
        if total != args.total_slots:
            raise ValueError(
                f"Warden total {total} does not match configured total {args.total_slots}"
            )
    except Exception as exc:
        used = args.total_slots
        print(f"Warden poll failed; reporting all slots consumed: {exc}", file=sys.stderr)
    if used != previous:
        update_last_consumed(args.sacctmgr, args.resource, used)
        print(
            f"reported Slurm license {args.resource} lastconsumed={used}",
            file=sys.stderr,
        )
    return used


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    """Parse reporter command-line arguments.

    Args:
        argv (Sequence[str] | None, optional): Arguments excluding the executable name.

    Returns:
        (argparse.Namespace): Parsed arguments.
    """
    parser = argparse.ArgumentParser()
    parser.add_argument("--warden-url", required=True)
    parser.add_argument("--resource", required=True)
    parser.add_argument("--total-slots", required=True, type=int)
    parser.add_argument("--timeout-seconds", type=float, default=3)
    parser.add_argument("--interval-seconds", type=float, default=5)
    parser.add_argument("--sacctmgr", default="sacctmgr")
    parser.add_argument("--once", action="store_true")
    args = parser.parse_args(argv)
    if args.total_slots < 1 or args.timeout_seconds <= 0 or args.interval_seconds <= 0:
        parser.error("slot total, timeout, and interval must be greater than zero")
    return args


def main(argv: Sequence[str] | None = None) -> int:
    """Run one report or the polling loop.

    Args:
        argv (Sequence[str] | None, optional): Arguments excluding the executable name.

    Returns:
        (int): Process exit status.
    """
    args = parse_args(argv)
    previous = None
    while True:
        previous = report_once(args, previous)
        if args.once:
            return 0
        time.sleep(args.interval_seconds)


if __name__ == "__main__":
    raise SystemExit(main())
