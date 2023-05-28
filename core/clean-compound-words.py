#!/usr/bin/env python3

# Read file from first argument of command line:
import sys


with open(sys.argv[1], "r", encoding="utf8") as f:
    words = set(f.read().splitlines())


def is_compound_word(word: str, words: set[str], depth: int = 0) -> bool:
    if depth > 0 and word in words:  # Only check if we're *not* the root word
        return True

    for i in range(1, len(word)):
        prefix = word[:i]
        suffix = word[i:]

        if prefix in words:
            return is_compound_word(
                # Probably a noun...
                suffix.title(),
                words,
                depth + 1,
            ) or is_compound_word(
                suffix,
                words,
                depth + 1,
            )

    return False


compound_words = set(word for word in words if is_compound_word(word, words))
non_compound_words = words - compound_words

sys.stdout.writelines(f"{word}\n" for word in sorted(non_compound_words))
sys.stderr.writelines(f"{word}\n" for word in sorted(compound_words))
