use std::collections::HashMap;
use std::rc::Rc;
use std::vec::Vec;

#[derive(Clone, Hash)]
struct State(u32, i32);

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for State {}

struct LevenshteinDfaState {
    offset: u32,
    states: Vec<State>,
}

struct LevenshteinDfa {
    dfa: HashMap<Vec<State>, HashMap<Vec<bool>, LevenshteinDfaState>>,
}

struct LevenshteinAutomaton {
    query: String,
    d: u8,
    // TODO: change to smart pointer that pyo3 respects
    dfa: Rc<LevenshteinDfa>,
}

struct LevenshteinAutomatonBuilder {
    d: u8,
    dfa: Rc<LevenshteinDfa>,
}

impl LevenshteinDfa {
    fn new(d: u8) -> Self {
        let mut dfa: HashMap<Vec<State>, HashMap<Vec<bool>, LevenshteinDfaState>> = HashMap::new();

        let state = Self::normalize(Self::initial_state(d));
        let char_vectors = Self::get_characteristic_vectors(2 * d + 1);

        dfa.insert(state.states.clone(), HashMap::new());
        let mut states_stack = vec![state.states];

        while states_stack.len() > 0 {
            let states = states_stack.pop().unwrap();
            let mut transitions: HashMap<Vec<bool>, LevenshteinDfaState> = HashMap::new();
            for vec in char_vectors.iter() {
                let next_state = Self::normalize(Self::step(vec, &states));
                if !dfa.contains_key(&next_state.states) {
                    dfa.insert(next_state.states.clone(), HashMap::new());
                    states_stack.push(next_state.states.clone());
                }

                transitions.insert(vec.clone(), next_state);
            }

            dfa.insert(states, transitions);
        }

        Self { dfa: dfa }
    }

    fn get_characteristic_vectors(width: u8) -> Vec<Vec<bool>> {
        fn create(vectors: Vec<Vec<bool>>, depth: u8, max: u8) -> Vec<Vec<bool>> {
            if depth == max {
                return vectors;
            }

            let mut new_vectors: Vec<Vec<bool>> = Vec::new();
            for v in vectors.into_iter() {
                new_vectors.push(vec![true].into_iter().chain(v.clone()).collect());
                new_vectors.push(vec![false].into_iter().chain(v).collect());
            }

            create(new_vectors, depth + 1, max)
        }

        let vectors = vec![vec![true], vec![false]];
        create(vectors, 1, width)
    }

    fn transitions(vector: &Vec<bool>, state: &State) -> Vec<State> {
        match &vector.iter().position(|x| *x == true) {
            Some(index) => {
                if *index == 1 {
                    return vec![State(state.0 + 1, state.1)];
                } else {
                    return vec![
                        State(state.0, state.1 - 1),
                        State(state.0 + 1, state.1 - 1),
                        State(state.0 + *index as u32 + 1, state.1 - *index as i32),
                    ];
                }
            }
            None => return vec![State(state.0, state.1 - 1), State(state.0 + 1, state.1 - 1)],
        }
    }

    fn step(vector: &Vec<bool>, states: &Vec<State>) -> Vec<State> {
        let mut next_states: Vec<State> = Vec::new();

        for s in states.iter() {
            let mut transitions: Vec<State> = Self::transitions(&vector, &s);
            while transitions.len() > 0 {
                let state = transitions.pop().unwrap();
                if state.1 >= 0 && !next_states.contains(&state) {
                    next_states.push(state);
                }
            }
        }

        next_states
    }

    fn initial_state(d: u8) -> Vec<State> {
        vec![State(0, d as i32)]
    }

    fn normalize(states: Vec<State>) -> LevenshteinDfaState {
        if states.len() == 0 {
            LevenshteinDfaState {
                offset: 0,
                states: Vec::new(),
            };
        }

        let min_offset =
            states.iter().fold(
                states[0].0,
                |offset, state| if offset < state.0 { offset } else { state.0 },
            );

        let mut states: Vec<State> = states
            .iter()
            .map(|s| State(s.0 - min_offset, s.1))
            .collect();

        states.sort_by(|s1, s2| s2.0.cmp(&s1.0));

        LevenshteinDfaState {
            offset: min_offset,
            states: states,
        }
    }
}

impl LevenshteinAutomaton {
    fn new(query: String, d: u8, dfa: Rc<LevenshteinDfa>) -> Self {
        Self {
            query: query,
            d: d,
            dfa: dfa,
        }
    }
}

impl LevenshteinAutomatonBuilder {
    fn new(d: u8) -> Self {
        Self {
            d: d,
            dfa: Rc::new(LevenshteinDfa::new(d)),
        }
    }

    fn get(&self, query: String) -> LevenshteinAutomaton {
        LevenshteinAutomaton::new(query, self.d, Rc::clone(&self.dfa))
    }
}
