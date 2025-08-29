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

    def _transitions(self, vec, s):
        offset, d = s
        yield (offset, d - 1)  # deletion
        yield (offset + 1, d - 1)  # substitution
        # insertion until character match
        for i, val in enumerate(vec[offset:]):
            if val:
                yield offset + i + 1, d - i

    def _is_state_redundant(self, s: tuple, state: tuple[tuple]):
        offset, d = s
        if d < 0:
            return True

        for s2 in state:
            offset_2, d_2 = s2
            # if state can be achived by changing characters between state2
            # with same or less modifications "d" it is redundant
            if s != s2 and d_2 - d >= abs(offset_2 - offset):
                return True

        return False

    def _remove_redundant_states(self, state: tuple[tuple]):
        """
        Remove all states that are redundant (used all modification steps,
        same outcome can be achived by replacing characters from other state)
        """

        return set(s for s in state if not self._is_state_redundant(s, state))

    def _step(self, vec: tuple[bool], state: tuple[tuple]):
        next_states = set()
        for s in state:
            # hashset union
            next_states |= set(self._transitions(vec, s))

        return self._remove_redundant_states(next_states)

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
