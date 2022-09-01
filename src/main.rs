mod db;
mod machine;
pub mod deciders;

fn main() {
    let (mut db, mut undecided_index) = db::load_default();

    println!("Checking index is sorted");
    undecided_index.assert_sorted();

    println!("Running cyclers decider - on full set");
    let undecided_time_count = db.header.undecided_time_count;
    let iter = (0.. undecided_time_count)
        .filter(|&id| {
            let description = db.read(id).expect("Failed to read machine");
            deciders::cyclers::decide(description)
        });
    let count = db::write_index(
        format!("cyclers-index-time-{}-maxIndex-{}",
            deciders::cyclers::MAX_STEPS,
            undecided_time_count,
        ),
        iter
    ).unwrap();
    println!("Decided {count}");
}
