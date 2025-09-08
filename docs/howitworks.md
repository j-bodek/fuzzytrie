This document briefly explains how the library works internally.
For clarity, a simplified version of the Rust implementation has been written in Python. It behaves the same conceptually, but without the performance optimizations.

For more details, see:
- Paper: [Fast string correction with Levenshtein automata Klaus U. Schulz, Stoyan Mihov](https://dmice.ohsu.edu/bedricks/courses/cs655/pdf/readings/2002_Schulz.pdf "Fast string correction with Levenshtein automata Klaus U. Schulz, Stoyan Mihov")
- Great blog post by Paul Mesurel: [Of Levenshtein automata implementations](https://fulmicoton.com/posts/levenshtein/ "Of levenshtein automata implementations")


### What is Levenshtein Automaton?

A Levenshtein automaton is a deterministic finite automaton ([DFA](https://en.wikipedia.org/wiki/Deterministic_finite_automaton "DFA")) that recognizes all words within a Levenshtein distance d from a given word.

In practice, this means that for any state there is defined exactly one transition for each possible input. By combining such an automaton with a trie, we can efficiently find all words in a set that are within distance **d** of a target query.


### How Does It Works?

The key idea is that we can build the DFA once for a given distance d and then reuse it for any word. This makes the automaton generic and independent of a specific query.

Core concepts
- Characteristic vector:
A binary vector that encodes the relationship between a character **c** and the target word **W**.
Example definition:
`g(W, c) = [1 if W[i] == c else 0 for i in range(len(W))]`

- Transitions:
Rules that define how the automaton moves from one state to another, given a characteristic vector.

Observations from paper:
- In definition 15 of the paper, it is stated that next state can be determined by the vector of subword M of length 2*d +1. Therefore, a charactaristic vector can be restricted to window of size 2*d + 1.
- The starting state is (0, d), where:
	- 0 = current position (offset) in the word
	- d = remaining allowed edits

Transition rules (simplified)
In paper transition rules are described in Table 2, the rules for state **(offset, d)** and characteristic vector V of width 2*d + 1:
- If V starts with 1:
	- Next state = ((offset+1, d))
- If V starts with 0 but contains 1 at position p:
	- Next state = ((offset, d-1), (offset+1, d-1), (offset+p+1, d-p))
- If V contains no 1:
	- Next state = ((offset, d-1), (offset+1, d-1))
	

### Constructing the Automaton

Using these rules, we can precompute all charactaristic vectors of size 2*d+1 and then iteratively expand the DFA starting from the initial state. This process is demonstrated in following snippet (implementation of each method can be found in [automaton.py](/docs/python/automaton.py "automaton.py"))

```
class LevenshteinDfa(dict):
    def __init__(self, d: int):
        super().__init__()

        _, state = self.normalize(self.initial_state(d))
        char_vectors = self._get_characteristic_vectors(2 * d + 1)

        self[state] = {}
        states_stack = [state]

        while states_stack:
            state = states_stack.pop()
            # compute next possible states for all characteristic vectors
            transitions = {}
            for vec in char_vectors:
                offset, next_state = self.normalize(self._step(vec, state))
                if next_state not in self:
                    self[next_state] = {}
                    states_stack.append(next_state)

                transitions[vec] = (offset, next_state)

            self[state] = transitions
```
