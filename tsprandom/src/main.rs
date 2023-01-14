use std::{
    sync::Mutex,
    thread::{self, available_parallelism},
};

use rustc_hash::{FxHashMap, FxHashSet};
use tsptools::{
    algorithms::hillclimb::hillclimb, helpers::random_solution, parsers::parse_tsp_file,
};

fn main() {
    let file = parse_tsp_file("./data/ulysses16.tsp").unwrap();

    const SAMPLE_COUNT: usize = 1000000;
    const MAX_RETRIES: usize = 100;

    let thread_count: usize = available_parallelism().unwrap().get();
    println!("{} threads available", thread_count);

    let samples_per_thread = SAMPLE_COUNT / thread_count;

    let distance_matrix = file.distance_matrix;
    let local_minimums = Mutex::new(FxHashMap::<Vec<usize>, (i32, i32)>::default());
    let visited_starting = Mutex::new(FxHashSet::default());

    thread::scope(|s| {
        for _ in 0..thread_count {
            s.spawn(|| {
                sample(
                    samples_per_thread,
                    MAX_RETRIES,
                    &distance_matrix,
                    &local_minimums,
                    &visited_starting,
                );
            });
        }
    });

    for e in visited_starting.into_inner().unwrap() {
        println!("{:?}", e);
    }
}

fn sample(
    sample_count: usize,
    max_retries: usize,
    distance_matrix: &Vec<Vec<i32>>,
    local_minimums: &Mutex<FxHashMap<Vec<usize>, (i32, i32)>>,
    visited_starting: &Mutex<FxHashSet<Vec<usize>>>,
) {
    for _ in 0..sample_count {
        let starting_solution = find_starting_point(visited_starting, distance_matrix, max_retries);
        if starting_solution.is_none() {
            return;
        }
        let starting_solution = starting_solution.unwrap();

        let (hillclimb_tour, hillclimb_len) = hillclimb(&starting_solution, distance_matrix);

        let mut map = local_minimums.lock().expect("Mutex poisoned, bailing out!");

        match map.get_mut(&hillclimb_tour) {
            Some(v) => {
                v.1 += 1;
            }
            None => {
                map.insert(hillclimb_tour, (hillclimb_len, 1));
            }
        }
    }
}

fn find_starting_point(
    visited_starting: &Mutex<FxHashSet<Vec<usize>>>,
    distance_matrix: &Vec<Vec<i32>>,
    max_retries: usize,
) -> Option<Vec<usize>> {
    let mut visited_set = visited_starting
        .lock()
        .expect("Mutex poisoned, bailing out!");

    let mut starting_solution = random_solution(distance_matrix.len(), None);
    let mut retries = 0;
    while visited_set.contains(&starting_solution) {
        retries += 1;
        if retries > max_retries {
            //can't find any new starting points, end thread
            return None;
        }
        starting_solution = random_solution(distance_matrix.len(), None);
    }

    visited_set.insert(starting_solution.clone());

    Some(starting_solution)
}
