import itertools
import dataclasses


@dataclasses.dataclass
class LevenshteinDfaState:
    offset: int
    state: tuple[tuple[int]]


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

    def _get_characteristic_vectors(self, width: int):
        # width of characteristic vector is equal to 2d + 1
        return list(itertools.product((True, False), repeat=width))

    def _get_index(self, vec, offset, val):
        try:
            return vec.index(val, offset) - offset
        except ValueError:
            return -1

    def _transitions(self, vec, s):
        offset, d = s

        if vec[offset] == 1:
            return ((offset + 1, d),)
        elif (index := self._get_index(vec, offset, True)) != -1:
            return (
                (offset, d - 1),
                (offset + 1, d - 1),
                (offset + index + 1, d - index),
            )
        else:
            return ((offset, d - 1), (offset + 1, d - 1))

    def _step(self, vec: tuple[bool], state: tuple[tuple]):
        next_states = set()
        for s in state:
            # hashset union
            next_states |= set(self._transitions(vec, s))

        # remove states with remaining edit distance less then 0
        return set(s for s in next_states if s[1] >= 0)

    def initial_state(self, d: int):
        return {(0, d)}

    def normalize(self, state: set[tuple]):
        if not state:
            return 0, ()

        min_offset = min(s[0] for s in state)
        state = tuple(
            sorted([(o - min_offset, d) for o, d in state], key=lambda x: x[0])
        )

        return min_offset, state


class LevenshteinAutomaton:
    def __init__(self, query: str, d: int, dfa: LevenshteinDfa):
        self.query = query
        self.d = d
        self.dfa = dfa
        self._characteristic_vector_cache = {}

    def _characteristic_vector(self, char: str, offset: int):
        key = (char, offset)
        if key not in self._characteristic_vector_cache:
            self._characteristic_vector_cache[key] = tuple(
                (
                    self.query[offset + i] == char
                    if offset + i < len(self.query)
                    else False
                )
                for i in range(2 * self.d + 1)
            )

        return self._characteristic_vector_cache[key]

    def initial_state(self):
        offset, state = self.dfa.normalize(self.dfa.initial_state(self.d))
        return LevenshteinDfaState(offset=offset, state=state)

    def step(self, char: str, state: LevenshteinDfaState):
        vec = self._characteristic_vector(char, state.offset)
        shift, next_state = self.dfa[state.state][vec]
        return LevenshteinDfaState(offset=state.offset + shift, state=next_state)

    def is_match(self, state: LevenshteinDfaState):
        for offset, d in state.state:
            if len(self.query) - (state.offset + offset) <= d:
                return True

        return False

    def can_match(self, state: LevenshteinDfaState):
        return len(state.state) > 0


class LevenshteinAutomatonBuilder:
    def __init__(self, d: int):
        self.d = d
        self.dfa = LevenshteinDfa(d)

    def get(self, query: str):
        return LevenshteinAutomaton(query, self.d, self.dfa)
