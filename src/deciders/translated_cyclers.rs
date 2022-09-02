use std::ops::Range;

use crate::machine::{Dir, Machine, MachineDescription, State};

pub const MAX_STEPS: usize = 10000;

/// Checks if there is a translated cycle
///
/// We do this by, as described in https://discuss.bbchallenge.org/t/decider-translated-cyclers/34
///
/// We execute the machine, while keeping a record of every time we "add" a new cell to the tape
/// (i.e. explore a cell we haven't previously explored). When that happens, we check if we've
/// previously added a cell while transitioning to the same state, and if the portion of the
/// tape we've seen since then is the same as the translated portion of the tape that existed
/// the last time we've done this.
///
/// If so, we're in a cycle, we're just going to keep adding on more of those translated segments.
pub fn decide(descr: MachineDescription) -> bool {
    let mut machine = Machine::new(descr);

    // This is a map from (Dir, State, Bit) to a list of stored snapshots from previous times
    // we added a new
    let breakpoint_set_index = |dir: Dir, state: State, bit: bool|
        dir as usize * 5 * 2 + (state as usize - 1) * 2 + bit as usize;
    let mut breakpoint_sets: [Vec<StoredSnapshot>; 20] = [(); 2 * 5 * 2].map(|_| Vec::new());

    let mut head_history = vec![];

    for step in 0..MAX_STEPS {
        head_history.push(machine.head());

        let transition = machine.transition();
        let new_state = transition.state().expect("Machine terminated");

        let is_breakpoint = (transition.dir == Dir::L && machine.head_offset == 0)
            || (transition.dir == Dir::R && machine.head_offset + 1 == machine.tape.len());

        if is_breakpoint {
            let bpi =  breakpoint_set_index(transition.dir, new_state, transition.out);
            let breakpoint_set = &mut breakpoint_sets[bpi];
            let current_snapshot = StoredSnapshot::new(step, &machine);

            // Check if we've already seen this position
            // TODO[perf]: Clever iteration order could make this substantially faster
            // (i.e. O(bp_count * step) to O(step)) by re-using leftmost/rightmost calculations
            for ref bp_snapshot in breakpoint_set.iter() {
                // Check if the tape from the farthest away cell modified, to the cell before the head
                // is the same at the previous breakpoint, and this breakpoint. If they are, we're
                // in a translated cycle.
                //
                // Note we don't need to check the cell containing the head, because we already
                // did that when we selected the breakpoint array.

                let prev_range;
                let current_range;
                match transition.dir {
                    Dir::R => {
                        let leftmost = head_history[bp_snapshot.step..]
                            .iter()
                            .copied()
                            .min()
                            .unwrap();
                        let delta = current_snapshot.head - leftmost;
                        prev_range = bp_snapshot.head - delta..bp_snapshot.head;
                        current_range = leftmost..current_snapshot.head;
                    }
                    Dir::L => {
                        let rightmost = head_history[bp_snapshot.step..]
                            .iter()
                            .copied()
                            .max()
                            .unwrap();
                        let delta = rightmost - current_snapshot.head;
                        prev_range = bp_snapshot.head + 1..bp_snapshot.head + 1 + delta;
                        current_range = current_snapshot.head + 1..rightmost + 1;
                    }
                }

                let previous_slice = bp_snapshot.tape_slice(prev_range);
                let current_slice = current_snapshot.tape_slice(current_range);
                assert!(current_slice.is_some());

                if previous_slice == current_slice {
                    return true;
                }
            }

            breakpoint_set.push(current_snapshot);
        }

        machine.step();
    }

    false
}

#[derive(Hash, PartialEq, Eq)]
struct StoredSnapshot {
    // Tape, as it was when the machine hit this step.
    step: usize,
    tape: Vec<bool>,
    state: State,
    head: i32,
    tape_start: i32,
}

impl StoredSnapshot {
    fn new(step: usize, machine: &Machine) -> Self {
        StoredSnapshot {
            step,
            tape: machine.tape.clone(),
            state: machine.state,
            head: machine.head(),
            tape_start: -(machine.cells_below_zero as i32),
        }
    }

    fn tape_slice(&self, idx: Range<i32>) -> Option<&[bool]> {
        // dbg!(&idx);
        let start = -self.tape_start + idx.start;
        let end = -self.tape_start + idx.end;

        if start < 0 || end as usize > self.tape.len() {
            return None;
        }
        Some(&self.tape[start as usize..end as usize])
    }
}

#[test]
fn test_positive_results() {
    let (mut db, _) = crate::db::load_default();
    let indices = [
        32510779, 45010518, 14427007, 14643029, 15167997, 50491158, 59645887, 31141863, 28690248,
    ];

    for index in indices {
        let descr = db.read(index).unwrap();

        println!("\n{index}");
        for row in descr.transitions {
            println!("{row:?}");
        }

        assert!(decide(descr));
    }
}

#[test]
fn test_large_positive_results() {
    let (mut db, _) = crate::db::load_default();
    let indices = [
        46965866, 74980673, 88062418, 59090563, 76989562, 46546554, 36091834, 58966114,
    ];

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
