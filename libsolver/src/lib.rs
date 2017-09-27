extern crate sudoku;

#[macro_use]
extern crate error_chain;

use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr;
use sudoku::Sudoku;

error_chain! {
    foreign_links {
        Utf8(::std::str::Utf8Error);
        Nul(::std::ffi::NulError);
    }
    errors {
        SolutionNotFound {
            description("solution for specified grid cannot be found")
        }
    }
}

pub extern "C" fn solve(sudoku_grid: *const c_char) -> *const c_char {
    assert!(!sudoku_grid.is_null());
    unsafe {
        let cstr = CStr::from_ptr(sudoku_grid);
        solve_ruster(cstr)
            .map(|result| result.into_raw() as *const _)
            .unwrap_or(ptr::null())
        // NOTE: CString is leaked here
    }
}

fn solve_ruster(sudoku_grid: &CStr) -> Result<CString> {
    let sudoku_grid_str = sudoku_grid.to_str()?;
    let solution = solve_rustest(sudoku_grid_str)?;

    Ok(CString::new(solution)?)
}

fn solve_rustest(sudoku_grid: &str) -> Result<String> {
    let sudoku = Sudoku::from_str(sudoku_grid).map_err(|e| e.to_string())?;

    match sudoku.solve_one() {
        Some(solution) => Ok(solution.to_string()),
        None => bail!(ErrorKind::SolutionNotFound),
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

        let solution = solve_rustest(sudoku_grid).unwrap();
        assert_eq!(solution, expected_solution);
    }
}
