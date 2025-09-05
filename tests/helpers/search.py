import time
from Levenshtein import distance


def get_lines(file):
    lines = []
    with open(file, "r+") as f:
        for l in f.readlines():
            lines.append(l.strip())

    return lines


def timeit(func, **kwargs):
    start = time.time()
    output = func(**kwargs)
    t = time.time() - start
    return t, output


def automaton_rs_matches(d, query, trie):
    return trie.search(d, query)


def brute_force_matches(query, words, d):
    matches = []

    for w in words:
        if distance(w, query) <= d:
            matches.append(w)

    return sorted(matches)
