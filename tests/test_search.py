import math
import pytest
from itertools import batched
from helpers.search import (
    get_lines,
    timeit,
    automaton_rs_matches,
    brute_force_matches,
)
from fuzzytrie_rs import FuzzyTrie


@pytest.fixture
def words():
    return get_lines("tests/assets/words.txt")


@pytest.fixture
def queries():
    return get_lines("tests/assets/queries.txt")


def test_automaton(queries, words):
    distances = [1, 2, 3]

    t_rs = FuzzyTrie()

    def insert_words(tr):
        for w in words:
            tr.add(w)

    t_rs_insert, _ = timeit(insert_words, tr=t_rs)
    print(f"\n\nInsertion of {len(words)} into rust trie took: {t_rs_insert}")

    for i, batch in enumerate(
        batched(queries, n=math.ceil(len(queries) / len(distances)))
    ):

        d = distances[i]
        s, _ = timeit(t_rs.init_automaton, d=d)
        print(f"\n\nInitialization of automaton with d={d} took: {s}")

        _time1, _time2 = 0, 0

        for q in batch:
            t1, a_matches = timeit(automaton_rs_matches, d=d, query=q, trie=t_rs)
            t2, bf_matches = timeit(brute_force_matches, query=q, words=words, d=d)
            _time1 += t1
            _time2 += t2

            assert sorted(a_matches) == sorted(
                bf_matches
            ), f"Returned matches differ d: {d}, query: {q}, automaton matches: {a_matches}, brute force matches: {bf_matches}"

        print(
            f"\nAverage time of levenshtein automaton search with d={d} {_time1 / len(batch)}"
            f"\nAverage time of brute force search with d={d} {_time2 / len(batch)}"
        )
