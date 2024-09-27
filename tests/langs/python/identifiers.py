"""Test for https://jackevans.bearblog.dev/refactoring-python-with-tree-sitter-jedi/
aka https://news.ycombinator.com/item?id=41637286
"""

import database
import pytest


@pytest.fixture()
def test_a(database):
    return database


def test_b(database):
    return database


database = "database"


class database:
    pass
