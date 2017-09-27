extern crate sudoku;

use error::*;
use self::sudoku::Sudoku;
use std::fmt;

// I don't want to implement sudoku solver here by myself, so I just 
// pick some crate that already does that job.

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
        let sudoku_grid = "\
___|2__|_63
3__|__5|4_1
__1|__3|98_
___|___|_9_
___|538|___
_3_|___|___
_26|3__|5__
5_3|7__|__8
47_|__1|___";

        let expected_solution = "
854 219 763
397 865 421
261 473 985

785 126 394
649 538 172
132 947 856

926 384 517
513 792 648
478 651 239";

        let mut solver = Solver::from_str(sudoku_grid).unwrap();
        let solution = solver.solve().unwrap();
        assert_eq!(solution, expected_solution);
    }
}
