use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_chacha::{self, ChaCha8Rng};
use getopt::Opt;
use std::{fs, io::Write};

fn fill_graph(distance_matrix:  &mut Vec<Vec<i32>>, density: f64, seed: Option<u64>, allow_loops: bool, is_directed: bool) {
    let mut rng = match seed {
        Some(seed) => ChaCha8Rng::seed_from_u64(seed),
        None => ChaCha8Rng::from_entropy(),
    };
    if density == 1.0 {
        for row in 0..distance_matrix.len() {
            for col in 0..distance_matrix[row].len() {
                if col != row {
                    distance_matrix[row][col] = Uniform::from(1..100).sample(&mut rng);
                    if is_directed {
                        distance_matrix[col][row] = distance_matrix[row][col];
                    }
                } else if allow_loops {
                    distance_matrix[row][col] = Uniform::from(1..100).sample(&mut rng);
                }
            }
        }
    } else {
        let mut edges = ((density * distance_matrix.len() as f64 * distance_matrix[0].len() as f64)) as i32;
        while edges > 0 {
            let row = Uniform::from(0..distance_matrix.len()).sample(&mut rng);
            let col = Uniform::from(0..distance_matrix[row].len()).sample(&mut rng);
            if distance_matrix[row][col] != 0  {
                continue;
            }
            let weight = Uniform::from(1..100).sample(&mut rng);
            if col != row {
                distance_matrix[row][col] = weight;
                if !is_directed {
                    distance_matrix[col][row] = weight;
                    edges -= 1;
                }
                edges -= 1;
            } else if allow_loops {
                distance_matrix[row][col] = weight;
                edges -= 1;
            }
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut opts = getopt::Parser::new(&args, "s:d:r:clo:h");
    
    let mut size = 0;
    let mut density = 0.0;
    let mut seed: Option<u64> = None;
    let mut allow_loops = false;
    let mut is_directed = false;
    let mut filename = None;
    let mut help = false;
    
    loop{
        match opts.next().transpose() {
            Ok(None) => break,
            Ok(Some(opt)) => match opt {
                Opt('s', Some(arg)) => size = arg.parse::<usize>().unwrap(),
                Opt('r', Some(arg)) => seed = Some(arg.parse::<u64>().unwrap()),
                Opt('c', _) => allow_loops = true,
                Opt('d', Some(arg)) => density = arg.parse::<f64>().unwrap(),
                Opt('l', _) => is_directed = true,
                Opt('o', Some(arg)) => filename = Some(arg),
                Opt('h', _) => help = true,
                _ => panic!("Invalid arguments"),
            }
            Err(_) => panic!("Invalid arguments"),
        }
    }
    if help {
        println!("Usage: tspgen -s <size> -d <density> [-r <seed>] [-c] [-l]");
        return;
    }
    if size == 0 || density == 0.0 {
        panic!("Usage: tspgen -s <size> -d <density> [-r <seed>] [-c] [-l]");
    }
    let mut distance_matrix = vec![vec![0; size]; size];
    fill_graph(&mut distance_matrix, density, seed, allow_loops, is_directed);

    if filename != None {
        let mut file = fs::File::create(filename.as_ref().unwrap()).unwrap();
        writeln!(file, "{}", size).unwrap();
        for row in 0..distance_matrix.len() {
            for col in 0..distance_matrix[row].len() {
                if col != distance_matrix[row].len() - 1 {
                    write!(file, "{}\t", distance_matrix[row][col]).unwrap();
                } else {
                    write!(file, "{}", distance_matrix[row][col]).unwrap();
                }
            }
            writeln!(file).unwrap();
        }
        println!("Graph written to {}", filename.as_ref().unwrap());
    } else {
        println!("{}", size);
        for row in 0..distance_matrix.len() {
            for col in 0..distance_matrix[row].len() {
                if col != distance_matrix[row].len() - 1 {
                    print!("{}\t", distance_matrix[row][col]);
                } else {
                    print!("{}", distance_matrix[row][col]);
                }
            }
            println!();
        }
    }
}