from automaton import LevenshteinAutomaton, LevenshteinDfaState
from dataclasses import dataclass, field
from typing import Self


@dataclass
class Node:
    end: bool = False  # indicate word end
    nodes: dict[str, Self] = field(default_factory=dict)


@dataclass
class Trie:
    nodes: dict[str, Node] = field(default_factory=dict)

    def add(self, word: str):
        nodes = self.nodes

        for i, c in enumerate(word):
            if c not in nodes:
                nodes[c] = Node()

            if len(word) - 1 == i:
                nodes[c].end = True

            nodes = nodes[c].nodes

    def delete(self, word: str):
        nodes, stack = self.nodes, [self.nodes]

        for i, c in enumerate(word):
            if c not in nodes:
                return

            # if end of word have childs only change that it isn't the word end
            # otherwise delete leafs
            if i == len(word) - 1:
                if nodes[c].nodes:
                    nodes[c].end = False
                    return
            else:
                stack.append(nodes[c].nodes)

            nodes = nodes[c].nodes

        # delete all leafs
        for c in word[::-1]:
            nodes = stack.pop()
            if nodes[c].nodes:
                break

            del nodes[c]

    def fuzzy_search(self, automaton: LevenshteinAutomaton):
        state = automaton.initial_state()
        for m in self._fuzzy_search(state, self.nodes, automaton):
            yield m

    def _fuzzy_search(
        self,
        state: LevenshteinDfaState,
        nodes: dict[str, Node],
        automaton: LevenshteinAutomaton,
    ):
        for char, node in nodes.items():
            new_state = automaton.step(char, state)
            # print(char, new_state)
            if not automaton.can_match(new_state):
                continue

            if node.end and automaton.is_match(new_state):
                yield char

            for m in self._fuzzy_search(new_state, node.nodes, automaton):
                yield char + m
