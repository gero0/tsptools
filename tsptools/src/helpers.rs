use std::{error::Error, fmt::Display};

use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: usize,
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub enum ParsingError {
    DimensionNotProvided,
    UnsupportedWeightFormat,
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionNotProvided => write!(f, "Parsing Error: no Dimension field",),
            Self::UnsupportedWeightFormat => write!(
                f,
                "Parsing Error: unsupported weight type or distance metric",
            ),
        }
    }
}

impl Error for ParsingError {}

pub fn generate_distance_matrix(
    nodes: &Vec<Node>,
    weight_type: &str,
) -> Result<Vec<Vec<i32>>, Box<dyn Error>> {
    let mut matrix = vec![vec![0; nodes.len()]; nodes.len()];

    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if i == j {
                continue;
            }
            matrix[i][j] = match weight_type {
                "EUC_2D" => dist2d_euc(&nodes[i], &nodes[j]),
                "CEIL_2D" => dist2d_euc_ceil(&nodes[i], &nodes[j]),
                "MAN_2D" => dist2d_man(&nodes[i], &nodes[j]),
                "MAX_2D" => dist2d_max(&nodes[i], &nodes[j]),
                "GEO" => dist_geo(&nodes[i], &nodes[j]),
                _ => return Err(Box::new(ParsingError::UnsupportedWeightFormat)),
            }
        }
    }

    Ok(matrix)
}

pub fn dist2d_euc(n1: &Node, n2: &Node) -> i32 {
    let xd = n2.x - n1.x;
    let yd = n2.y - n1.y;
    (xd * xd + yd * yd).sqrt() as i32
}

pub fn dist2d_euc_ceil(n1: &Node, n2: &Node) -> i32 {
    let xd = n2.x - n1.x;
    let yd = n2.y - n1.y;
    let len = 0.5 + (xd * xd + yd * yd).sqrt();
    len as i32
}

pub fn dist2d_man(n1: &Node, n2: &Node) -> i32 {
    let xd = (n2.x - n1.x).abs();
    let yd = (n2.y - n1.y).abs();
    (xd + yd) as i32
}

pub fn dist2d_max(n1: &Node, n2: &Node) -> i32 {
    let xd = (n2.x - n1.x).abs();
    let yd = (n2.y - n1.y).abs();
    xd.max(yd) as i32
}

pub fn dist_geo(n1: &Node, n2: &Node) -> i32 {
    let rrr = 6378.388;
    let (x1, y1) = (to_rad(n1.x), to_rad(n1.y));
    let (x2, y2) = (to_rad(n2.x), to_rad(n2.y));

    let q1 = (y1 - y2).cos();
    let q2 = (x1 - x2).cos();
    let q3 = (x1 + x2).cos();

    (rrr * (0.5 * ((1.0 + q1) * q2 - (1.0 - q1) * q3)).acos() + 1.0) as i32
}

pub fn to_rad(x: f32) -> f32 {
    let pi = 3.141592;
    let deg = x as i32;
    let min = x - deg as f32;

    pi * (deg as f32 + 5.0 * min / 3.0) / 180.0
}

pub fn tour_len(path: &Vec<usize>, distance_matrix: &Vec<Vec<i32>>) -> i32 {
    let len: i32 = path.windows(2).map(|w| distance_matrix[w[0]][w[1]]).sum();
    len + distance_matrix[path[0]][path[path.len() - 1]]
}

pub fn random_solution(node_count: usize, seed: Option<u64>) -> Vec<usize> {
    let mut nodes_remaining: Vec<usize> = (0..node_count).collect();
    let mut starting_path = vec![];

    let mut rng = match seed {
        Some(seed) => ChaCha8Rng::seed_from_u64(seed),
        None => ChaCha8Rng::from_entropy(),
    };

    while !nodes_remaining.is_empty() {
        let between = Uniform::from(0..nodes_remaining.len());
        let i = between.sample(&mut rng);
        starting_path.push(nodes_remaining[i]);
        nodes_remaining.remove(i);
    }

    starting_path
}

pub fn nodes_to_ids(path: &[Node]) -> Vec<usize> {
    return path.iter().map(|node| node.pos).collect();
}

pub fn cmp_permutations(perm1: &[usize], perm2: &[usize]) -> u32 {
    //invert first permutation
    let mut perm_1_inv = perm1.to_owned();
    for i in 0..perm1.len() {
        perm_1_inv[perm1[i]] = i;
    }

    //Compose the two permutations
    let mut p = vec![0; perm1.len()];
    for i in 0..perm1.len() {
        p[i] = perm2[perm_1_inv[i]];
    }

    let mut count = 0;
    for i in 0..perm1.len() {
        while p[i] != i {
            let a = p[p[i]];
            let b = p[i];
            p.swap(a, b);
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::cmp_permutations;

    #[test]
    fn perm_cmp_test() {
        let perm_1 = [0, 1, 2, 3, 4, 5];
        let perm_2 = [5, 4, 3, 2, 1, 0];

        let result = cmp_permutations(&perm_1, &perm_2);
        assert_eq!(result, 3);

        let perm_1 = [5, 4, 3, 2, 1, 0];
        let perm_2 = [5, 4, 3, 2, 1, 0];

        let result = cmp_permutations(&perm_1, &perm_2);
        assert_eq!(result, 0);

        let perm_1 = [5, 4, 1, 2, 3, 0];
        let perm_2 = [5, 4, 3, 2, 1, 0];

        let result = cmp_permutations(&perm_1, &perm_2);
        assert_eq!(result, 1);

        let perm_1 = [0, 2, 4, 1, 3, 5];
        let perm_2 = [0, 1, 2, 3, 4, 5];

        let result = cmp_permutations(&perm_1, &perm_2);
        assert_eq!(result, 3);
    }
}
