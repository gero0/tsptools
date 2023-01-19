use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_chacha::{self, ChaCha8Rng};

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