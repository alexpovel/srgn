#!/usr/bin/env python3

import argparse
import re
import subprocess
from difflib import unified_diff
from pathlib import Path

ENCODING = "utf8"


def diff(a: str, b: str, filename: Path) -> str:
    res = "\n".join(
        unified_diff(
            a.splitlines(),
            b.splitlines(),
            fromfile=str(filename),
            tofile=str(filename),
        )
    )
    return res


def update_markdown_file(file: Path) -> None:
    content = file.read_text(encoding=ENCODING)
    print(f"Got contents of {file}, {len(content)} long")
    pattern = re.compile(
        pattern=r"(?<=```console\n\$ srgn --help\n)(.*?)(?=```$)",
        flags=re.DOTALL | re.MULTILINE,
    )
    args = ["cargo", "run", "--", "--help"]
    result = subprocess.run(
        args=args,
        capture_output=True,
        text=True,
        check=True,
    )
    print(f"Successfully ran command: {args}")

    match_ = re.search(pattern, content)
    assert match_ is not None, "Bad regex/README"
    print(f"Match at: {match_}")

    new_content = re.sub(pattern, result.stdout, content)

    print("Got new file contents. Diff:")
    print(diff(content, new_content, filename=file))
    input("Press Enter to confirm and write to file, CTRL+C to abort...")
    file.write_text(new_content, encoding=ENCODING)
    print("Wrote new file contents.")


def main():
    parser = argparse.ArgumentParser(
        description="Update Markdown console code blocks with actual command output"
    )
    parser.add_argument("file", help="Path to the Markdown file to process")
    args = parser.parse_args()

    file = Path(args.file)
    update_markdown_file(file)
    print(f"Successfully updated {file}")
    print("Done")


if __name__ == "__main__":
    main()
