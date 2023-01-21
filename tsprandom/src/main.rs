use std::{
    env,
    fs::File,
    io::Write,
    process::Command,
    sync::Mutex,
    thread::{self, available_parallelism},
};

use rustc_hash::{FxHashMap, FxHashSet};
use tsptools::{
    algorithms::{hillclimb::hillclimb, two_opt::two_opt},
    helpers::{cmp_permutations, random_solution},
    parsers::parse_tsp_file,
};

use plotters::prelude::*;

mod simpleparser;

type HillclimbFunction = dyn Fn(&Vec<usize>, &Vec<Vec<i32>>, bool) -> (Vec<usize>, i32);

fn main() {
    let path = env::args().nth(1).expect("No path to input data given!");
    if path == "--help" || path == "-h" || path == "help" {
        println!("Usage: tsprandom <path to tsp file> <algorithm> [sample_count (default 10000)] [max_retries (default 10000)]");
        println!("Supported algorithms: hc, 2opt");
        return;
    }

    let (alg, sample_count, max_retries) = get_args();

    let distance_matrix = if path.ends_with(".txt") {
        simpleparser::parse_simple(&path).expect("Could not parse input file")
    } else {
        let file = parse_tsp_file(&path).unwrap();
        file.distance_matrix
    };

    let algorithm = match alg.as_str() {
        "hc" => hillclimb,
        "2opt" => two_opt,
        _ => panic!("Invalid algorithm param"),
    };

    let thread_count: usize = available_parallelism().unwrap().get();
    println!("{} threads available", thread_count);

    let samples_per_thread = sample_count / thread_count;

    let local_minimums = Mutex::new(FxHashMap::<Vec<usize>, (i32, i32)>::default());
    let visited_starting = Mutex::new(FxHashSet::default());

    thread::scope(|s| {
        for _ in 0..thread_count {
            s.spawn(|| {
                sample(
                    samples_per_thread,
                    max_retries,
                    &distance_matrix,
                    &local_minimums,
                    &visited_starting,
                    &algorithm,
                );
            });
        }
    });

    let local_minimums = local_minimums.into_inner().unwrap();
    let visited_starting = visited_starting.into_inner().unwrap();

    let mut local_minimums: Vec<_> = local_minimums
        .into_iter()
        .map(|(k, v)| (k, v.0, v.1))
        .collect();
    local_minimums.sort_by(|a, b| a.1.cmp(&b.1));

    println!("Calculating stats");
    calculcate_stats(&local_minimums, &alg);

    println!("Saving results...");
    save_results(&local_minimums, &visited_starting, &alg);
}

fn get_args() -> (String, usize, usize) {
    let alg = env::args()
        .nth(2)
        .expect("Algorithm param required (hc or 2opt)");
    let sample_count: usize = match env::args().nth(3) {
        Some(n) => n.parse().expect("Invalid sample count argument"),
        None => 10000,
    };

    let max_retries: usize = match env::args().nth(4) {
        Some(n) => n.parse().expect("Invalid max_retries argument"),
        None => 10000,
    };

    (alg, sample_count, max_retries)
}

fn sample(
    sample_count: usize,
    max_retries: usize,
    distance_matrix: &Vec<Vec<i32>>,
    local_minimums: &Mutex<FxHashMap<Vec<usize>, (i32, i32)>>,
    visited_starting: &Mutex<FxHashSet<Vec<usize>>>,
    hc: &HillclimbFunction,
) {
    for _ in 0..sample_count {
        let starting_solution = find_starting_point(visited_starting, distance_matrix, max_retries);
        if starting_solution.is_none() {
            return;
        }
        let starting_solution = starting_solution.unwrap();

        let (hillclimb_tour, hillclimb_len) = (hc)(&starting_solution, distance_matrix, true);

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

    let mut starting_solution = random_solution(distance_matrix.len(), None, true);
    let mut retries = 0;
    while visited_set.contains(&starting_solution) {
        retries += 1;
        if retries > max_retries {
            //can't find any new starting points, end thread
            return None;
        }
        starting_solution = random_solution(distance_matrix.len(), None, true);
    }

    visited_set.insert(starting_solution.clone());

    Some(starting_solution)
}

fn save_results(
    local_minimums: &Vec<(Vec<usize>, i32, i32)>,
    visited_starting: &FxHashSet<Vec<usize>>,
    alg_name: &str,
) {
    let dt = chrono::offset::Local::now().to_string();
    let lopath = format!("{}_local_optima_{}.csv", alg_name, dt);
    let sppath = format!("{}_starting_points_{}.csv", alg_name, dt);

    let mut lo_file = File::create(&lopath).expect("Could not create file!");
    let mut starting_points_file = File::create(&sppath).expect("Could not create file!");

    starting_points_file.write("tour\n".as_bytes()).unwrap();
    for e in visited_starting {
        starting_points_file
            .write_fmt(format_args!("{:?}\n", e))
            .unwrap();
    }

    lo_file
        .write("id;tour;tour_len;related_starting_points\n".as_bytes())
        .unwrap();
    for (i, (tour, len, sp)) in local_minimums.iter().enumerate() {
        lo_file
            .write_fmt(format_args!("{};{:?};{};{}\n", i, tour, *len, *sp))
            .unwrap();
    }

    let output = Command::new("python3")
        .args(["vis.py", &lopath, "local_optima_hillclimb_graph"])
        .output();
    match output {
        Ok(output) => match output.status.success() {
            true => println!("Plot saved!"),
            false => println!(
                "Python drawing module returned an error!: {:?}",
                output.stdout
            ),
        },
        Err(_) => println!("Failed to run python process!"),
    }
}

fn calculcate_stats(local_minimums: &Vec<(Vec<usize>, i32, i32)>, alg_name: &str) {
    //calculate distances from node to best node and height differences between them
    let mut distances = vec![0; local_minimums.len() - 1];
    let mut height_diff = vec![0; local_minimums.len() - 1];

    let best = &local_minimums[0];
    for i in 1..local_minimums.len() {
        distances[i - 1] = cmp_permutations(&best.0, &local_minimums[i].0) as u64;
        height_diff[i - 1] = (local_minimums[i].1 - best.1) as u64;
    }

    println!("Plotting...");
    plot_corr(&distances, &height_diff, alg_name);

    //expected values
    let ed = distances.iter().sum::<u64>() / distances.len() as u64;
    let eh = height_diff.iter().sum::<u64>() / height_diff.len() as u64;
    //expected value of products of the two variables
    let ep = distances
        .iter()
        .zip(&height_diff)
        .map(|(a, b)| a * b)
        .sum::<u64>()
        / distances.len() as u64;
    //calculate covariance
    let cov = ep - (ed * eh);

    let mean_d_squared = distances.iter().map(|x| x * x).sum::<u64>() / distances.len() as u64;
    let mean_h_squared = height_diff.iter().map(|x| x * x).sum::<u64>() / height_diff.len() as u64;

    let std_d = ((mean_d_squared - (ed * ed)) as f64).sqrt();
    let std_h = ((mean_h_squared - (eh * eh)) as f64).sqrt();

    let cor = cov as f64 / (std_d * std_h);

    println!("Mean distance:{}, Mean height difference:{}", ed, eh);
    println!(
        "std dev. of distance:{}, std dev. of height diff:{}",
        std_d, std_h
    );
    println!("Covariance: {}\nCorrelation:{}", cov, cor);
}

fn plot_corr(distances: &Vec<u64>, height_diff: &Vec<u64>, alg_name: &str) {
    let dt = chrono::offset::Local::now().to_string();
    let path = format!("{}_corr_{}.png", alg_name, dt);
    let root_area = BitMapBackend::new(&path, (800, 400)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let max_height = height_diff[height_diff.len() - 1] + 100;
    let max_dist = *distances.iter().max().unwrap() + 2;
    let mut min_dist = *distances.iter().min().unwrap();

    if min_dist >= 2{
        min_dist -= 2;
    }

    let x_range = min_dist..max_dist as u64;
    let y_range = 0..max_height as u64;

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(x_range, y_range)
        .unwrap();

    ctx.configure_mesh()
        .x_desc("Distance form best solution (in pair swaps)")
        .y_desc("Path length difference from best solution")
        .draw()
        .unwrap();

    ctx.draw_series(
        distances
            .iter()
            .zip(height_diff)
            .map(|(x, y)| Circle::new((*x, *y), 5, &BLUE)),
    )
    .unwrap();

    root_area.present().unwrap();
}
