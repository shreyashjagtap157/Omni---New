#!/usr/bin/env python3
"""Compatibility entry point for the canonical strict inventory validator."""

from pathlib import Path
import runpy
import sys

strict_validator = Path(__file__).with_name("validate-inventory-strict.py")
sys.argv[0] = str(strict_validator)
runpy.run_path(str(strict_validator), run_name="__main__")
