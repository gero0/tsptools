use crate::helpers::*;

pub fn two_opt_random(distance_matrix: &Vec<Vec<i32>>, seed: Option<u64>) -> (Vec<usize>, i32) {
    let starting_tour = random_solution(distance_matrix.len(), seed);
    two_opt(&starting_tour, distance_matrix)
}

pub fn two_opt(starting_tour: &Vec<usize>, distance_matrix: &Vec<Vec<i32>>) -> (Vec<usize>, i32) {
    let mut tour = starting_tour.to_owned();
    let n = tour.len();
    let mut improvement = true;

    while improvement {
        improvement = false;
        let mut min_dist = 0;
        let mut a = 0;
        let mut b = 0;

        for i in 0..(n - 1) {
            for j in (i + 1)..n {
                let distance = distance_matrix[tour[i]][tour[j]]
                    + distance_matrix[tour[i + 1]][tour[(j + 1) % n]]
                    - distance_matrix[tour[i]][tour[i + 1]]
                    - distance_matrix[tour[j]][tour[(j + 1) % n]];

                if distance < min_dist {
                    min_dist = distance;
                    a = i;
                    b = j;
                    improvement = true;
                }
            }
        }

        if !improvement {
            break;
        }

        //reverse [a+1, b]
        a += 1;
        while a < b {
            tour.swap(a, b);
            a += 1;
            b -= 1;
        }
    }

    let len = tour_len(&tour, distance_matrix);
    (tour, len)
}

#[test]
fn rev_test() {
    let mut a = 2;
    let mut b = 5;
    let mut v = vec![1, 2, 3, 4, 5, 6];
    while a < b {
        v.swap(a, b);
        a += 1;
        b -= 1;
    }

    assert_eq!(v, vec![1, 2, 6, 5, 4, 3]);
}
