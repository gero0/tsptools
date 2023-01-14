use crate::helpers::*;
use std::{error::Error, fmt::Display, fs};

#[derive(Debug, Clone)]
pub struct TspFile {
    pub name: String,
    pub dimension: usize,
    pub distance_matrix: Vec<Vec<i32>>,
}

pub fn parse_tsp_file(path: &str) -> Result<TspFile, Box<dyn Error>> {
    let file = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let mut dimension = None;
    let mut name = None;
    let mut edge_wf = "FUNCTION";
    let mut edge_wt = "EUC_2D";

    let mut weights_i = 0;

    loop {
        let line = lines[weights_i].trim();
        if line == "NODE_COORD_SECTION" || line == "EDGE_WEIGHT_SECTION" {
            weights_i += 1;
            break;
        }

        let (key, val) = parse_line(line);

        match key {
            "DIMENSION" => dimension = Some(val.parse::<u32>()?),
            "NAME" => name = Some(val),
            "EDGE_WEIGHT_FORMAT" => edge_wf = val,
            "EDGE_WEIGHT_TYPE" => edge_wt = val,
            _ => {}
        }

        weights_i += 1;
    }

    //return error if we don't have a dimension provided
    let dimension = dimension.ok_or(ParsingError::DimensionNotProvided)? as usize;
    let name = String::from(name.unwrap_or(""));

    let distance_matrix = match edge_wf {
        "FUNCTION" => parse_nodelist(&lines[weights_i..weights_i + dimension], edge_wt)?,
        "FULL_MATRIX" => parse_full_matrix(&lines[weights_i..weights_i + dimension])?,
        // "UPPER_ROW" => parse_half_matrix(
        //     &lines[weights_i..weights_i + dimension - 1],
        //     dimension,
        //     true,
        //     false,
        // )?,
        // "UPPER_DIAG_ROW" => parse_half_matrix(
        //     &lines[weights_i..weights_i + dimension],
        //     dimension,
        //     true,
        //     true,
        // )?,
        // "LOWER_ROW" => parse_half_matrix(
        //     &lines[weights_i..weights_i + dimension - 1],
        //     dimension,
        //     false,
        //     false,
        // )?,
        // "LOWER_DIAG_ROW" => parse_half_matrix(
        //     &lines[weights_i..weights_i + dimension],
        //     dimension,
        //     false,
        //     true,
        // )?,
        _ => return Err(Box::new(ParsingError::UnsupportedWeightFormat)),
    };

    Ok(TspFile {
        name,
        dimension,
        distance_matrix,
    })
}

fn parse_line(line: &str) -> (&str, &str) {
    let tokens: Vec<&str> = line.split(':').collect();
    let key = tokens[0].trim();
    let val = tokens[1].trim();
    (key, val)
}

fn parse_nodelist(lines: &[&str], weight_type: &str) -> Result<Vec<Vec<i32>>, Box<dyn Error>> {
    let mut nodes = vec![];

    //parse nodes
    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let id: u32 = tokens[0].trim().parse()?;
        let x: f32 = tokens[1].trim().parse()?;
        let y: f32 = tokens[2].trim().parse()?;
        nodes.push(Node { pos: i, id, x, y });
    }

    generate_distance_matrix(&nodes, weight_type)
}

fn parse_full_matrix(lines: &[&str]) -> Result<Vec<Vec<i32>>, Box<dyn Error>> {
    let mut d_matrix = vec![];
    for line in lines {
        let mut row = vec![];
        let line = line.trim();
        let tokens = line.split_whitespace();
        for token in tokens {
            row.push(token.parse::<i32>()?)
        }
        d_matrix.push(row);
    }

    Ok(d_matrix)
}

fn parse_half_matrix(
    lines: &[&str],
    dim: usize,
    upper: bool,
    diag: bool,
) -> Result<Vec<Vec<i32>>, Box<dyn Error>> {
    let mut d_matrix = vec![vec![0; dim]; dim];

    for (row, line) in lines.iter().enumerate() {
        let line = line.trim();
        let tokens = line.split_whitespace();
        for (i, token) in tokens.enumerate() {
            let offset_x = match upper {
                true => row,
                false => dim - 1 - row,
            };
            let offset_y = match diag {
                true => row + i,
                false => row + i + 1,
            };
            d_matrix[offset_x][offset_y] = token.parse::<i32>()?
        }
    }

    Ok(d_matrix)
}

pub fn parse_tour_file(path: &str) -> Result<Vec<u32>, Box<dyn Error>> {
    let file = fs::read_to_string(path).unwrap();
    let mut lines = file.lines();

    for line in lines.by_ref() {
        let line = line.trim();
        if line == "TOUR_SECTION" {
            break;
        }
    }

    let mut path = vec![];

    //parse path
    for line in lines {
        let line = line.trim();
        if line == "-1" {
            break;
        }
        let id: u32 = line.parse()?;
        path.push(id - 1);
    }

    Ok(path)
}
