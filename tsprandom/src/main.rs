use std::{
    fs::File,
    io::Write,
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

    let local_minimums = Mutex::new(FxHashMap::<Vec<usize>, (i32, i32)>::default());
    let visited_starting = Mutex::new(FxHashSet::default());

    thread::scope(|s| {
        for _ in 0..thread_count {
            s.spawn(|| {
                sample(
                    samples_per_thread,
                    MAX_RETRIES,
                    &file.distance_matrix,
                    &local_minimums,
                    &visited_starting,
                );
            });
        }
    });

    save_results(
        &local_minimums.into_inner().unwrap(),
        &visited_starting.into_inner().unwrap(),
    );
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

fn save_results(
    local_minimums: &FxHashMap<Vec<usize>, (i32, i32)>,
    visited_starting: &FxHashSet<Vec<usize>>,
) {
    let mut lo_file = File::create("local_optime.txt").expect("Could not create file!");
    let mut starting_points_file =
        File::create("starting_points.txt").expect("Could not create file!");

    starting_points_file.write("tour\n".as_bytes()).unwrap();
    for e in visited_starting {
        starting_points_file
            .write_fmt(format_args!("{:?}\n", e))
            .unwrap();
    }

    lo_file
        .write("tour;tour_len;related_starting_points\n".as_bytes())
        .unwrap();
    for (tour, (len, sp)) in local_minimums {
        lo_file
            .write_fmt(format_args!("{:?};{};{}\n", tour, len, sp))
            .unwrap();
    }
}
