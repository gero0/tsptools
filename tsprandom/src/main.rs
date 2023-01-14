use std::{
    sync::{Arc, Mutex},
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
    let visited_starting_points = Mutex::new(FxHashSet::default());

    thread::scope(|s| {
        for _ in 0..thread_count {
            s.spawn(|| {
                for _ in 0..samples_per_thread {
                    let mut visited_set = visited_starting_points
                        .lock()
                        .expect("Mutex poisoned, bailing out!");

                    let mut starting_solution = random_solution(distance_matrix.len(), None);
                    let mut retries = 0;
                    while visited_set.contains(&starting_solution) {
                        retries += 1;
                        if retries > MAX_RETRIES {
                            //can't find any new starting points, end thread
                            return;
                        }
                        starting_solution = random_solution(distance_matrix.len(), None);
                    }

                    visited_set.insert(starting_solution.clone());

                    //drop the lock to release the mutex
                    drop(visited_set);

                    let (hillclimb_tour, hillclimb_len) =
                        hillclimb(&starting_solution, &distance_matrix);

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
            });
        }
    });

    for e in visited_starting_points.lock().unwrap().iter() {
        println!("{:?}", e);
    }
}
