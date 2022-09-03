use std::mem;

#[allow(dead_code)] // False positive
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Dir {
    R = 0,
    L = 1,
}

#[allow(dead_code)] // False positive
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum NewState {
    Undef = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum State {
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Transition {
    pub out: bool,
    pub dir: Dir,
    pub new_state: NewState,
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MachineDescription {
    pub transitions: [Transition; 10],
}

impl State {
    fn index(self) -> usize {
        self as usize - 1
    }
}

impl NewState {
    fn state(self) -> Option<State> {
        match self {
            NewState::Undef => None,
            NewState::A => Some(State::A),
            NewState::B => Some(State::B),
            NewState::C => Some(State::C),
            NewState::D => Some(State::D),
            NewState::E => Some(State::E),
        }
    }

    fn from_state_idx(idx: usize) -> Self {
        match idx {
            0 => NewState::A,
            1 => NewState::B,
            2 => NewState::C,
            3 => NewState::D,
            4 => NewState::E,
            _ => unreachable!(),
        }
    }

    fn from_state(state: State) -> NewState {
        match state {
            State::A => NewState::A,
            State::B => NewState::B,
            State::C => NewState::C,
            State::D => NewState::D,
            State::E => NewState::E,
        }
    }
}

impl MachineDescription {
    pub fn from_bytes(bytes: &[u8]) -> &MachineDescription {
        assert_eq!(bytes.len(), 30);
        assert_eq!(30, mem::size_of::<MachineDescription>());
        assert_eq!(1, mem::align_of::<MachineDescription>());

        unsafe { &*(bytes.as_ptr() as *const MachineDescription) }
    }

    pub fn transition(&self, state: State, cell: bool) -> Transition {
        let transition_index = (state as usize - 1) * 2 + cell as usize;
        self.transitions[transition_index]
    }

    pub fn set_transition(&mut self, state: State, cell: bool, transition: Transition) {
        let transition_index = (state as usize - 1) * 2 + cell as usize;
        self.transitions[transition_index] = transition;
    }

    pub fn has_less_than_5_states(&self) -> bool {
        let mut map = [None; 5];
        let mut map_next = 0;

        fn make_map_recuirsive(map: &mut [Option<NewState>; 5], map_next: &mut usize, original: &MachineDescription, state: State) {
            if map.iter().any(|&x| x == Some(NewState::from_state(state))) {
                return
            }

            map[*map_next] = Some(NewState::from_state(state));
            *map_next += 1;

            if let Some(next_state) = original.transition(state, false).state() {
                make_map_recuirsive(map, map_next, original, next_state)
            }
            if let Some(next_state) = original.transition(state, true).state() {
                make_map_recuirsive(map, map_next, original, next_state)
            }
        };

        make_map_recuirsive(&mut map, &mut map_next, self, State::A);

        map_next != 5
    }

    pub fn normalize(&self) -> MachineDescription {
        // println!("");
        // Map new state -> old state
        let mut map = [None; 5];
        let mut map_next = 0;

        fn make_map_recuirsive(map: &mut [Option<NewState>; 5], map_next: &mut usize, original: &MachineDescription, state: State) {
            if map.iter().any(|&x| x == Some(NewState::from_state(state))) {
                return
            }

            map[*map_next] = Some(NewState::from_state(state));
            *map_next += 1;

            if let Some(next_state) = original.transition(state, false).state() {
                make_map_recuirsive(map, map_next, original, next_state)
            }
            if let Some(next_state) = original.transition(state, true).state() {
                make_map_recuirsive(map, map_next, original, next_state)
            }
        };

        make_map_recuirsive(&mut map, &mut map_next, self, State::A);

        if map_next != 5 {
            // println!("map {map:?}");
            // for row in self.transitions {
            //     println!("{:?}", row);
            // }

            // Some states are actually unreachable, it's fine to replace them with Undef.
            // TODO: Consider some method of filtering these instead?
            // They happen when we're constructing simplified machines.
            for i in 0.. 5 {
                if map[i] == None {
                    map[i] = Some(NewState::Undef)
                }
            }
        }
        // TMP
        // assert_eq!(map_next, 5, "Wow, some state is unreachable in {map:?}");


        let mut transitions = std::array::from_fn(|i| {
            let new_state_idx = i / 2;
            let old_state = map[new_state_idx];
            if let Some(old_state) = old_state.and_then(|x| x.state()) {
                let bit = i % 2 == 1;
                let old_transition = self.transition(old_state, bit);
                let new_transition_state = old_transition.state().map(|transit_state| {
                    map[transit_state.index()].unwrap()
                }).unwrap_or(NewState::Undef);
                Transition {
                    new_state: new_transition_state,
                    .. old_transition
                }
            } else {
                Transition {
                    new_state: NewState::Undef,
                    out: false,
                    dir: Dir::L,
                }
            }

        });

        if transitions[0].dir == Dir::R {
            for transition in &mut transitions {
                transition.dir = match transition.dir {
                    Dir::R => Dir::L,
                    Dir::L => Dir::R,
                }
            }
        }

        // println!("new");
        // for row in transitions {
        //     println!("{:?}", row);
        // }

        MachineDescription { transitions }
    }
}

pub struct Machine {
    pub description: MachineDescription,
    pub head_offset: usize,
    pub cells_below_zero: usize,
    pub state: State,
    pub tape: Vec<bool>,
}

impl Transition {
    pub fn state(&self) -> Option<State> {
        self.new_state.state()
    }
}

impl Machine {
    pub fn new(description: MachineDescription) -> Self {
        Machine {
            description,
            head_offset: 0,
            cells_below_zero: 0,
            state: State::A,
            tape: vec![false],
        }
    }

    pub fn head(&self) -> i32 {
        self.head_offset as i32 - self.cells_below_zero as i32
    }

    pub fn transition(&self) -> Transition {
        self.description
            .transition(self.state, self.tape[self.head_offset])
    }

    /// Returns true if the machine has halted
    pub fn step(&mut self) -> bool {
        let transition = self.transition();
        self.state = match transition.state() {
            Some(s) => s,
            None => return true,
        };

        self.tape[self.head_offset] = transition.out;
        match (self.head_offset, transition.dir) {
            (0, Dir::L) => {
                self.tape.insert(0, false);
                self.cells_below_zero += 1;
            }
            (_, Dir::L) => self.head_offset -= 1,
            (_, Dir::R) => {
                if self.head_offset + 1 == self.tape.len() {
                    self.tape.push(false);
                }
                self.head_offset += 1;
            }
        }

            false
    }

    #[allow(dead_code)]
    pub fn tape_str(&self, left_padding: usize) -> String {
        if left_padding < self.cells_below_zero {
            panic!("Out of space to print");
        }

        let rem_padding = left_padding - self.cells_below_zero;
        let mut out = String::new();
        for _ in 0..rem_padding {
            out.push('_');
        }
        for (i, val) in self.tape.iter().copied().enumerate() {
            if i == self.head_offset {
                let mut state_char = format!("{:?}", self.state);
                if !val {
                    state_char = state_char.to_lowercase();
                }
                out.push_str(&state_char)
            } else {
                if val {
                    out.push('■')
                } else {
                    out.push('□')
                }
            }
        }
        out
    }
}
