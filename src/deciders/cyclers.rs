use std::collections::HashSet;

use crate::machine::{Machine, MachineDescription, State};

pub const MAX_STEPS: usize = 1_000;

/// Detects machines that visit the exact same state (tape + head + machine state)
/// twice, and are therefore in a loop. Does so in the stupidest possible manner,
/// just keeps a set of all the states we've seen before and sees if the new one
/// is in it.
pub fn decide(description: MachineDescription) -> bool {
    let mut set = HashSet::new();

    let mut machine = Machine::new(description);
    for _ in 0..=MAX_STEPS {
        if !set.insert(StoredState::from(&machine)) {
            // Set already contianed the state
            return true;
        }

        machine.step();
    }

    false
}

#[derive(Hash, PartialEq, Eq)]
struct StoredState {
    // Tape, starting at the first non-false value.
    tape: Vec<bool>,
    state: State,
    head: i32,
    tape_start: i32,
}

impl From<&Machine> for StoredState {
    fn from(machine: &Machine) -> Self {
        let leading_zeros = machine.tape.iter().take_while(|&&x| !x).count();
        // This can't overflow, because MAX_STEPS is significantly below 2^30
        // and both of these grow/shrink at most 1 per step.
        let tape_start = leading_zeros as i32 - machine.cells_below_zero as i32;
        let tape = machine.tape[leading_zeros..].to_vec();
        // Can't overflow, as above.
        let head = machine.head_offset as i32 - machine.cells_below_zero as i32;

        StoredState {
            tape,
            state: machine.state,
            head,
            tape_start,
        }
    }
}

#[test]
fn test_finds_cyclers() {
    let (mut db, _) = crate::db::load_default();
    let indices = [11636047, 4231819, 279081];

    for index in indices {
        let descr = db.read(index).unwrap();
        assert!(decide(descr));
    }
}

#[test]
fn test_negative_results() {
    let (mut db, _) = crate::db::load_default();
    let indices = [
        14017021, 13206000, 8107478, 14053644, 14276172, 78082807, 83293270, 1201055, 9354848,
        6369968, 5795478, 12745999, 13578663, 23400034,
    ];

    for index in indices {
        let descr = db.read(index).unwrap();
        assert!(!decide(descr));
    }
}
