mod db;
mod machine;

fn main() {
    let mut db = db::Db::open("all_5_states_undecided_machines_with_global_header")
        .expect("Failed to open db");

    let mut undecided_index = db::Index::open("bb5_undecided_index").expect("Failed to open index");

    for index in undecided_index.iter().take(5) {
        let description = db.read(index).expect("Failed to read machine");

        for transition in description.transitions {
            println!("{transition:?}");
        }

        let mut machine = machine::Machine::new(description);

        println!("{:?} {}", machine.state, machine.tape_str(10));
        for _ in 0..10 {
            if machine.step() {
                println!("Halted");
                break;
            }
            println!("{:?} {}", machine.state, machine.tape_str(10));
        }
    }
}
