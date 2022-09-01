mod db;
mod machine;
pub mod deciders;

fn main() {
    let (mut db, mut undecided_index) = db::load_default();

    println!("Checking index is sorted");
    undecided_index.assert_sorted();

    println!("Running cyclers decider - on undecided set");
    let undecided_time_count = db.header.undecided_time_count;
    for index in undecided_index.iter().take_while(|&idx| idx < undecided_time_count) {
        let description = db.read(index).expect("Failed to read machine");

        if deciders::cyclers::decide(description) {
            panic!("Unexpected new decision");
        }
    }

    println!("Running cyclers decider - on full set");
    let mut count = 0;
    for index in 0.. db.header.undecided_time_count {
        let description = db.read(index).expect("Failed to read machine");

        if deciders::cyclers::decide(description) {
            count += 1;
        }
    }
    println!("Decided {count}");

    // println!("Running something");
    // for index in undecided_index.iter().take(5) {
    //     let description = db.read(index).expect("Failed to read machine");

    //     for transition in description.transitions {
    //         println!("{transition:?}");
    //     }

    //     let mut machine = machine::Machine::new(description);

    //     println!("{:?} {}", machine.state, machine.tape_str(10));
    //     for _ in 0..10 {
    //         if machine.step() {
    //             println!("Halted");
    //             break;
    //         }
    //         println!("{:?} {}", machine.state, machine.tape_str(10));
    //     }
    // }
}
