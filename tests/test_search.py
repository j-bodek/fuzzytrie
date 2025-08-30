import pytest
from helpers.search import get_lines, timeit, automaton_matches, brute_force_matches
from src import LevenshteinAutomatonBuilder, Trie


@pytest.fixture
def words():
    return get_lines("tests/assets/words.txt")


@pytest.fixture
def queries():
    return get_lines("tests/assets/queries.txt")


def test_automaton(queries, words):
    # queries = queries[:10]
    # edit distance = 2
    d = 2
    builder = LevenshteinAutomatonBuilder(d)
    t = Trie()
    _time1, _time2 = 0, 0

    for w in words:
        t.add(w)

    for q in queries:
        t1, a_matches = timeit(automaton_matches, query=q, builder=builder, trie=t)
        t2, bf_matches = timeit(brute_force_matches, query=q, words=words, d=d)
        _time1 += t1
        _time2 += t2

        assert (
            a_matches == bf_matches
        ), f"Returned matches differ, automaton matches: {a_matches}, brute force matches: {bf_matches}"

    print(
        f"\nAverage time of levenshtein automaton search {_time1 / len(queries)}"
        f"\nAverage time of brute force search {_time2 / len(queries)}"
    )
