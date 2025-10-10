use nohash_hasher::BuildNoHashHasher;
use std::cmp;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
struct State(u32, i32);

pub struct LevenshteinDfaState {
    offset: u32,
    max_shift: u32,
    state_id: u32,
}

struct LevenshteinDfa {
    dfa: HashMap<
        u32,
        HashMap<u32, LevenshteinDfaState, BuildNoHashHasher<u32>>,
        BuildNoHashHasher<u32>,
    >,
    states: HashMap<u32, Vec<State>, BuildNoHashHasher<u32>>,
}

pub struct LevenshteinAutomaton {
    query_len: u32,
    d: u8,
    dfa: Arc<LevenshteinDfa>,
    characteristic_vector_cache: HashMap<usize, Vec<u32>, BuildNoHashHasher<usize>>,
}

pub struct LevenshteinAutomatonBuilder {
    d: u8,
    dfa: Arc<LevenshteinDfa>,
}

impl LevenshteinDfa {
    fn new(d: u8) -> Self {
        let mut dfa: HashMap<
            u32,
            HashMap<u32, LevenshteinDfaState, BuildNoHashHasher<u32>>,
            BuildNoHashHasher<u32>,
        > = HashMap::default();
        let mut states_map: HashMap<u32, Vec<State>, BuildNoHashHasher<u32>> = HashMap::default();

        let (_, _, states) = Self::initial_state(d);
        let char_vectors = Self::get_characteristic_vectors(2 * d + 1);

        // map states vector to corresponding numerical id
        let mut states_ids: HashMap<Vec<State>, u32> = HashMap::new();
        let state_id = Self::get_states_id(&states, &mut states_ids);
        dfa.insert(state_id, HashMap::default());
        states_map.insert(state_id, states.clone());

        let mut states_stack = vec![states];

        while states_stack.len() > 0 {
            let states = states_stack.pop().unwrap();
            let mut transitions: HashMap<u32, LevenshteinDfaState, BuildNoHashHasher<u32>> =
                HashMap::default();

            for vec in char_vectors.iter() {
                let (offset, max_shift, next_states) = Self::normalize(Self::step(vec, &states));
                let next_state_id = Self::get_states_id(&next_states, &mut states_ids);

                if !dfa.contains_key(&next_state_id) {
                    dfa.insert(next_state_id, HashMap::default());
                    states_stack.push(next_states);
                }

                transitions.insert(
                    Self::vec_to_mask(vec),
                    LevenshteinDfaState {
                        offset: offset,
                        max_shift: max_shift,
                        state_id: next_state_id,
                    },
                );
            }

            let state_id = Self::get_states_id(&states, &mut states_ids);
            dfa.insert(state_id, transitions);
            states_map.insert(state_id, states);
        }

        Self {
            dfa: dfa,
            states: states_map,
        }
    }

    fn get_states_id(states: &Vec<State>, states_ids: &mut HashMap<Vec<State>, u32>) -> u32 {
        // Map vector of states into corresponding numerical id

        if states.len() == 0 {
            return 0;
        }

        match states_ids.get(states) {
            Some(id) => *id,
            None => {
                let id = (states_ids.len() + 1) as u32;
                states_ids.insert(states.clone(), id);
                id
            }
        }
    }

    fn get_characteristic_vectors(width: u8) -> Vec<Vec<u8>> {
        // Return all characteristic vectors of width 'width'

        fn create(vectors: Vec<Vec<u8>>, depth: u8, max: u8) -> Vec<Vec<u8>> {
            if depth == max {
                return vectors;
            }

            let mut new_vectors: Vec<Vec<u8>> = Vec::new();
            for v in vectors.into_iter() {
                new_vectors.push(v.clone().into_iter().chain(vec![1]).collect());
                new_vectors.push(v.into_iter().chain(vec![0]).collect());
            }

            create(new_vectors, depth + 1, max)
        }

        let vectors = vec![vec![1], vec![0]];
        create(vectors, 1, width)
    }

    fn transitions(vector: &Vec<u8>, state: &State) -> Vec<State> {
        // Perform all possible state transitions and return them

        match &vector[state.0 as usize..vector.len()]
            .iter()
            .position(|x| *x == 1)
        {
            Some(index) => {
                if *index as u32 == 0 {
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

    fn step(vector: &Vec<u8>, states: &Vec<State>) -> Vec<State> {
        // Perform step from 'states' step for specifiec characteristic vector

        let mut next_states: Vec<State> = Vec::new();

        for s in states.iter() {
            for state in Self::transitions(&vector, &s) {
                if state.1 >= 0 && !next_states.contains(&state) {
                    next_states.push(state);
                }
            }
        }

        next_states
    }

    fn vec_to_mask(vec: &[u8]) -> u32 {
        // builds bitmask from binary vector
        // vector is vector of 0 and 1 and can be represented as single numeric value

        let mut mask = 0u32;
        for (i, &b) in vec.iter().enumerate() {
            if b != 0 {
                mask |= 1 << i;
            }
        }

        mask
    }

    fn initial_state(d: u8) -> (u32, u32, Vec<State>) {
        // return offset, max_shift and vector of states
        Self::normalize(vec![State(0, d as i32)])
    }

    fn normalize(states: Vec<State>) -> (u32, u32, Vec<State>) {
        // return offset, max_shift and vector of states
        if states.len() == 0 {
            return (0, 0, Vec::new());
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

        states.sort_by(|s1, s2| s1.cmp(&s2));

        let max_shift = states
            .iter()
            .fold(states[0].0 as i32 + states[0].1, |offset, state| {
                if offset > state.0 as i32 + state.1 {
                    offset
                } else {
                    state.0 as i32 + state.1
                }
            });

        (min_offset, max_shift as u32, states)
    }
}

impl LevenshteinAutomaton {
    fn new(query: String, d: u8, dfa: Arc<LevenshteinDfa>) -> Self {
        Self {
            characteristic_vector_cache: Self::create_characteristic_vector_cache(&query, d),
            query_len: query.chars().count() as u32,
            d: d,
            dfa: dfa,
        }
    }

    fn create_characteristic_vector_cache(
        query: &str,
        d: u8,
    ) -> HashMap<usize, Vec<u32>, BuildNoHashHasher<usize>> {
        // Creates cache of vector bit maps, maps character and specific offset to corresponding vector bitmap

        let mut cache: HashMap<usize, Vec<u32>, BuildNoHashHasher<usize>> = HashMap::default();
        for c in query.chars() {
            let mut char_vec: Vec<u8> = query
                .chars()
                .map(|ch| if ch == c { 1 } else { 0 })
                .collect();
            // create bitmask for vectors
            char_vec.append(&mut vec![0; 2 * d as usize + 1]);

            let mut char_vec_masks: Vec<u32> = vec![];
            let mut mask = 0u32;
            let window = (2 * d + 1) as usize;

            for (i, &b) in char_vec.iter().enumerate() {
                if i + 1 > window {
                    // shift bits to right
                    mask = mask >> 1;
                }

                if b != 0 {
                    mask |= 1 << cmp::min(window - 1, i);
                }

                if i + 1 >= window {
                    char_vec_masks.push(mask);
                }
            }

            cache.insert(c as usize, char_vec_masks);
        }

        cache
    }

    pub fn initial_state(&self) -> LevenshteinDfaState {
        // returns starting state
        LevenshteinDfaState {
            offset: 0,
            max_shift: self.d as u32,
            state_id: 1,
        }
    }

    pub fn step(&mut self, c: char, state: &LevenshteinDfaState) -> LevenshteinDfaState {
        // performs single automaton step
        let vec = self.get_characteristic_vector(c, state.offset);

        match self.dfa.as_ref().dfa.get(&state.state_id) {
            Some(transitions) => match transitions.get(&vec) {
                Some(next_state) => LevenshteinDfaState {
                    offset: state.offset + next_state.offset,
                    max_shift: next_state.max_shift,
                    state_id: next_state.state_id,
                },
                None => LevenshteinDfaState {
                    offset: 0,
                    max_shift: 0,
                    state_id: 0,
                },
            },
            _ => LevenshteinDfaState {
                offset: 0,
                max_shift: 0,
                state_id: 0,
            },
        }
    }

    pub fn is_match(&self, state: &LevenshteinDfaState) -> bool {
        self.query_len as i32 - state.offset as i32 <= state.max_shift as i32
    }

    pub fn can_match(&self, state: &LevenshteinDfaState) -> bool {
        state.state_id != 0
    }

    pub fn distance(&self, state: &LevenshteinDfaState) -> u16 {
        match self.dfa.as_ref().states.get(&state.state_id) {
            Some(states) => {
                states
                    .iter()
                    .map(|s| {
                        self.d as i32 - s.1 as i32
                            + cmp::max(0, self.query_len as i32 - state.offset as i32 - s.0 as i32)
                    })
                    .fold(
                        // distance is only called on matched words so it's save to use 'd' as upper bound
                        self.d as u16,
                        |d1, d2| if d1 < d2 as u16 { d1 } else { d2 as u16 },
                    )
            }
            _ => self.query_len as u16,
        }
    }

    fn get_characteristic_vector(&self, c: char, offset: u32) -> u32 {
        // return charactaristic vector for specific character and offset

        match self.characteristic_vector_cache.get(&(c as usize)) {
            Some(vec_masks) => return vec_masks[offset as usize],
            None => return 0,
        }
    }
}

impl LevenshteinAutomatonBuilder {
    pub fn new(d: u8) -> Self {
        Self {
            d: d,
            dfa: Arc::new(LevenshteinDfa::new(d)),
        }
    }

    pub fn get(&self, query: String) -> LevenshteinAutomaton {
        LevenshteinAutomaton::new(query, self.d, Arc::clone(&self.dfa))
    }
}
