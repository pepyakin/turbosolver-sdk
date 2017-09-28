//! Solver itself. You can't change the grid after Solver was created.

// I don't want to implement sudoku solver here by myself, so I just
// pick some crate that already does the job.

extern crate sudoku;

use error::*;
use self::sudoku::Sudoku;
use std::fmt;

pub struct Solver(Sudoku);

impl Solver {
    pub fn from_str(grid: &str) -> Result<Solver> {
        let sudoku = Sudoku::from_str(grid).map_err(ParseError)?;
        Ok(Solver(sudoku))
    }

    pub fn solve(&mut self) -> Option<String> {
        self.0.solve_one().map(|x| x.to_string())
    }
}

/// Wrapper type of sudoku::ParseError.
///
/// It is needed because sudoku::ParseError doesn't implement Error trait.
#[derive(Clone, Debug)]
pub struct ParseError(self::sudoku::ParseError);

impl ::std::error::Error for ParseError {
    fn description(&self) -> &str {
        use self::sudoku::ParseError::*;
        match self.0 {
            InvalidLineLength(..) => "line contains more/less than 9 digits",
            InvalidNumber(..) => "grid contains invalid digit",
            NotEnoughRows => "grid contains more/less than 9 lines",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let sudoku_grid = include_str!("sudoku.txt");
        let expected_solution = include_str!("sudoku_solution.txt");

        let mut solver = Solver::from_str(sudoku_grid).unwrap();
        let solution = solver.solve().unwrap();
        assert_eq!(solution, expected_solution);
    }
}
