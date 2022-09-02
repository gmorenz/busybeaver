mod db;
pub mod deciders;
mod machine;

fn main() {
    run_cyclers_translated();
}

fn run_cyclers_translated() {
    let (mut db, mut undecided_index) = db::load_default();

    println!("Checking index is sorted");
    undecided_index.assert_sorted();

    println!("Running cyclers translated");
    let iter = undecided_index.iter().filter(|&id| {
        let description = db.read(id).expect("Failed to read machine");
        deciders::cyclers::decide(description)
    });
    let count = db::write_index("cyclers-translated-index", iter).unwrap();
    println!("Decided {count}");
}

#[allow(dead_code)]
fn run_cyclers() {
    let (mut db, mut undecided_index) = db::load_default();

    println!("Checking index is sorted");
    undecided_index.assert_sorted();

    println!("Running cyclers decider - on full set");
    let undecided_time_count = db.header.undecided_time_count;
    let iter = (0..undecided_time_count).filter(|&id| {
        let description = db.read(id).expect("Failed to read machine");
        deciders::cyclers::decide(description)
    });
    let count = db::write_index(
        format!(
            "cyclers-index-time-{}-maxIndex-{}",
            deciders::cyclers::MAX_STEPS,
            undecided_time_count,
        ),
        iter,
    )
    .unwrap();
    println!("Decided {count}");
}
