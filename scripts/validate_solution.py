#!/usr/bin/env python
"""Validate a Rust solution against a benchmark problem via cargo test."""
import json
import os
import subprocess
import sys
import tempfile
import shutil

def validate(slug, solution, test_code, cargo_toml=""):
    """Returns (passed: bool, error: str)."""
    test_dir = os.path.join(tempfile.gettempdir(), f"opus_validate_{slug}")

    # Clean up previous
    if os.path.exists(test_dir):
        shutil.rmtree(test_dir, ignore_errors=True)

    os.makedirs(os.path.join(test_dir, "src"), exist_ok=True)
    os.makedirs(os.path.join(test_dir, "tests"), exist_ok=True)

    # Cargo.toml
    if not cargo_toml:
        cargo_toml = f'[package]\nname = "{slug}"\nversion = "0.1.0"\nedition = "2021"\n'

    with open(os.path.join(test_dir, "Cargo.toml"), "w") as f:
        f.write(cargo_toml)

    with open(os.path.join(test_dir, "src", "lib.rs"), "w") as f:
        f.write(solution)

    # Test file name: slug with hyphens → underscores
    test_filename = slug.replace("-", "_") + ".rs"
    with open(os.path.join(test_dir, "tests", test_filename), "w") as f:
        f.write(test_code)

    # Run cargo test
    try:
        result = subprocess.run(
            ["cargo", "test"],
            cwd=test_dir,
            capture_output=True,
            text=True,
            timeout=60,
            env={**os.environ, "CARGO_TARGET_DIR": os.path.join(tempfile.gettempdir(), "opus_validate_target")},
        )
        output = result.stdout + result.stderr
        passed = result.returncode == 0 and "test result: ok" in output
        return passed, "" if passed else output[-500:]
    except subprocess.TimeoutExpired:
        return False, "timeout"
    except Exception as e:
        return False, str(e)
    finally:
        shutil.rmtree(test_dir, ignore_errors=True)

if __name__ == "__main__":
    # Usage: python validate_solution.py <problem.json> <solution.rs>
    with open(sys.argv[1]) as f:
        problem = json.load(f)
    with open(sys.argv[2]) as f:
        solution = f.read()

    passed, error = validate(problem["slug"], solution, problem["test_code"], problem.get("cargo_toml", ""))
    if passed:
        print(f"PASS: {problem['slug']}")
    else:
        print(f"FAIL: {problem['slug']}")
        print(error)
    sys.exit(0 if passed else 1)
