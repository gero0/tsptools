use std::{error::Error, fmt::Display, fs};

#[derive(Debug)]
struct ParseError;
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parsing Error")
    }
}
impl Error for ParseError {}

fn parse_simple(path: &str) -> Result<Vec<Vec<i32>>, Box<dyn Error>> {
    let file = fs::read_to_string("address.txt")?;
    let mut lines = file.lines();
    let n: usize = lines.next().ok_or(ParseError)?.parse()?;

    let mut distance_matrix = vec![vec![0; n]; n];

    for i in 0..n {
        let line = lines.next().ok_or(ParseError)?;
        let tokens: Vec<_> = line.split_whitespace().collect();
        for j in 0..n {
            let val: i32 = tokens[j].parse()?;
            distance_matrix[i][j] = val;
        }
    }

    Ok(distance_matrix)
}
