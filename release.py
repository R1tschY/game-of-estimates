#!/usr/bin/env python3
import dataclasses
import os
import re
import subprocess
from argparse import ArgumentParser
from pathlib import Path
from typing import Tuple, TypeAlias

ROOT = Path(__file__).parent

Version: TypeAlias = Tuple[int, int, int]


@dataclasses.dataclass(frozen=True)
class Version:
    major: int
    minor: int
    patch: int

    def __str__(self):
        return f"{self.major}.{self.minor}.{self.patch}"


def get_version() -> Version:
    v = re.search(
        r'version\s*=\s*"(\d+)\.(\d+)\.(\d+)"',
        (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    return Version(int(v.group(1)), int(v.group(2)), int(v.group(3)))


def inc_version(v: Version, step: str):
    if step == "major":
        return Version(v.major + 1, v.minor, v.patch)
    elif step == "minor":
        return Version(v.major, v.minor + 1, v.patch)
    elif step == "patch":
        return Version(v.major, v.minor, v.patch + 1)


def replace_version(path: Path, old_version: Version, new_version: Version):
    path.write_text(
        path.read_text(encoding="utf-8")
        .replace(str(old_version), str(new_version)),
        encoding="utf-8")


def main():
    argparser = ArgumentParser()
    argparser.add_argument("step", choices=["major", "minor", "patch"])
    args = argparser.parse_args()

    release_version = get_version()
    next_version = inc_version(release_version, args.step)

    subprocess.check_call([
        "git", "tag", "-a", f"v{release_version}", "-m", f"Release {release_version}"])
    subprocess.check_call([
        "git", "push", "origin", f"v{release_version}"])

    replace_version(Path(ROOT / "Cargo.toml"), release_version, next_version)
    replace_version(Path(ROOT / "frontend" / "package.json"), release_version, next_version)
    subprocess.check_call([
        "git", "commit", "-m", f"Bump version to {next_version}", "--",
        os.fspath(Path(ROOT / "Cargo.toml")),
        os.fspath(Path(ROOT / "Cargo.lock")),
        os.fspath(Path(ROOT / "frontend" / "package.json"))])
    subprocess.check_call([
        "git", "push", "origin"])


if __name__ == '__main__':
    main()
