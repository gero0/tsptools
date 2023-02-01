use crate::helpers::{random_solution, tour_len};

pub fn hillclimb_rand(
    distance_matrix: &Vec<Vec<i32>>,
    seed: Option<u64>,
    preserve_first: bool,
) -> (Vec<u16>, i32) {
    let random_tour = random_solution(distance_matrix.len() as u16, seed, preserve_first);
    hillclimb(&random_tour, distance_matrix, preserve_first)
}

pub fn hillclimb(
    starting_tour: &Vec<u16>,
    distance_matrix: &Vec<Vec<i32>>,
    preserve_first: bool,
) -> (Vec<u16>, i32) {
    let mut current_tour = starting_tour.clone();
    let mut current_len = tour_len(&current_tour, distance_matrix);

    loop {
        let neighbors = get_neighbors(&current_tour, preserve_first);
        let (best_neighbor, best_neighbor_len) = get_best_neighbor(&neighbors, distance_matrix);

        if best_neighbor_len >= current_len {
            break;
        }
        current_tour = best_neighbor;
        current_len = best_neighbor_len;
    }

    (current_tour, current_len)
}

fn get_neighbors(path: &Vec<u16>, preserve_first: bool) -> Vec<Vec<u16>> {
    let mut neighbors = vec![];

    let start = match preserve_first {
        true => 1,
        false => 0,
    };

    for i in start..path.len() {
        for j in i + 1..path.len() {
            let mut neighbor = path.clone();
            neighbor.swap(i, j);
            neighbors.push(neighbor);
        }
    }

    neighbors
}

fn get_best_neighbor(
    neighbors: &Vec<Vec<u16>>,
    distance_matrix: &Vec<Vec<i32>>,
) -> (Vec<u16>, i32) {
    let mut best_len = tour_len(&neighbors[0], distance_matrix);
    let mut best_neighbor_index = 0;

    for (i, neighbor) in neighbors[1..].iter().enumerate() {
        let len = tour_len(neighbor, distance_matrix);
        if len < best_len {
            best_len = len;
            best_neighbor_index = i + 1;
        }
    }

    (neighbors[best_neighbor_index].clone(), best_len)
}
