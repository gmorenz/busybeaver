use std::collections::HashSet;

use crate::db::Index;

mod db;
pub mod deciders;
mod machine;

fn main() {
    // Primitive method of choosing what to run by modifying main
    run_cyclers();
    run_cyclers_translated();
}


#[allow(dead_code)]
/// Regenerates a specific translated-cyclers run, exactly.
///
/// Our implementation takes more steps to find a cycle than the reference implementation sometimes,
/// so if a machine is in the reference set and we don't get it after the same number of steps, we
/// try twice as many.
fn run_cyclers_translated() {
    let (mut db, _) = db::load_default();

    let mut expected_output = Index::open("translated-cyclers-run-8f5b2539279a-time-1000-space-500-minIndex-14322029-maxIndex-88664064")
        .expect("Failed to open official tranlsted cyclers index");
    let expected_set = expected_output.iter().collect::<HashSet<_>>();

    let start_idx = db.header.undecided_time_count;

    println!("Running cyclers translated");

    let mut tried_more_on = 0;
    let iter = db.size_index().into_iter().filter(|&id| {
        let description = db.read(id).expect("Failed to read machine");
        let mut res = deciders::translated_cyclers::decide::<1000>(description.clone());
        if !res && expected_set.contains(&id) {
            tried_more_on += 1;
            res = deciders::translated_cyclers::decide::<2000>(description.clone());
        }

        assert_eq!(expected_set.contains(&id), res);
        res
    });
    let count = db::write_index(
        format!("cyclers-translated-index-time-{}-minIndex-{start_idx}", 1000),
        iter
    ).unwrap();
    println!("Decided {count}, had to use more steps on {tried_more_on}");
}

#[allow(dead_code)]
fn run_cyclers() {
    let (mut db, mut undecided_index) = db::load_default();

    println!("Checking index is sorted");
    undecided_index.assert_sorted();

    println!("Running cyclers decider - on machines that ran out of time");
    let undecided_time_count = db.header.undecided_time_count;
    let iter = db.time_index().into_iter().filter(|&id| {
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
