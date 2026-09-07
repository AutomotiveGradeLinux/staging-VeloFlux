#!/usr/bin/env python3
# Copyright 2026 EMQ Technologies Co., Ltd.
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.


import sys
from pathlib import Path

if __package__ is None:
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from agents.nl2pipeline.cli.langgraph import main  # noqa: E402


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

