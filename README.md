## FuzzyTrie

**FuzzyTrie** is a lightweight, fast, and dependency-free fuzzy search library for Python, written in Rust. It performs efficient string matching and correction using a Levenshtein automaton, as described in [this paper](https://dmice.ohsu.edu/bedricks/courses/cs655/pdf/readings/2002_Schulz.pdf "paper")

### Usage

```
from fuzzytrie import FuzzyTrie

t = FuzzyTrie()

# Initialize automatons for edit distances of 1 and 2
t.init_automaton(d=1)
t.init_automaton(d=2)

for w in ["lavanshtain", "levanshtain", "levenshtain"]:
    t.add(w)

# Perform fuzzy searches with different maximum distances
print(t.search(query="levenshtein", d=2))
# Returns list of distances between query and element, and elements
# >> [(2, 'levanshtain'), (1, 'levenshtain')]
print(t.search(query="levenshtein", d=1))
# >> [(1, 'levenshtain')]
```


### Limitations
- The maximum supported Levenshtein distance is 15.
- Automaton initialization time grows quickly with larger distances.
	For example:
	- d = 1–3: initialization completes in a fraction of a second.
	- d = 4: initialization takes about 14 seconds.


### Benchmarks
Benchmarks were run on a dataset of ~460k words with 500 sample queries for each distance. Results were compared against a brute-force implementation (Levenshtein distance in C from [rapidfuzz/Levenshtein](https://github.com/rapidfuzz/Levenshtein "library"))


| Distance | Automaton init time (s) | Avg. FuzzyTrie search (s) | Avg. brute force search (s) | Speedup |
| --- |:---:|:---:|:---:|:---|
| 1 | ~3.29e-05 | ~0.00046 | ~0.14 | 306x |
| 2 | ~0.000608 | ~0.00167 | ~0.14 | 84x |
| 3 | ~0.053419 | ~0.00502 | ~0.14 | 27x |


### Theory of operation
Brief explaination of how this works internally can be found [here](/docs/howitworks.md "here")