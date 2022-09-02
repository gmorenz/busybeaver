use std::mem;

#[allow(dead_code)] // False positive
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir {
    R = 0,
    L = 1,
}

#[allow(dead_code)] // False positive
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
pub struct Transition {
    pub out: bool,
    pub dir: Dir,
    pub new_state: NewState,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct MachineDescription {
    pub transitions: [Transition; 10],
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
        match self.new_state {
            NewState::Undef => None,
            NewState::A => Some(State::A),
            NewState::B => Some(State::B),
            NewState::C => Some(State::C),
            NewState::D => Some(State::D),
            NewState::E => Some(State::E),
        }
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
        self.description.transition(self.state, self.tape[self.head_offset])
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
