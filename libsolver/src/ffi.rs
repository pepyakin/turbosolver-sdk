use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr;

use solver::*;
use error::*;

#[no_mangle]
pub extern "C" fn solver_create(sudoku_grid: *const c_char, f: fn(*mut Solver, *const c_char)) {
    assert!(!sudoku_grid.is_null(), "sudoku_grid should not be null");

    fn solver_create_inner(sudoku_grid: *const c_char) -> Result<Box<Solver>> {
        unsafe {
            let cstr = CStr::from_ptr(sudoku_grid).to_str()?;
            let solver = Solver::from_str(cstr)?;
            Ok(Box::new(solver))
        }
    }

    match solver_create_inner(sudoku_grid) {
        Ok(solver) => f(Box::into_raw(solver), ptr::null()),
        Err(e) => {
            // Mind that `c_err_str` must outlive `f` call!
            let c_err_str =
                CString::new(e.description()).expect("e.description() should be valid cstring");
            f(ptr::null_mut(), c_err_str.as_ptr());
        }
    }
}

#[no_mangle]
pub extern "C" fn solver_solve(solver: *mut Solver, f: fn(*const c_char)) {
    unsafe {
        let solver = solver.as_mut().expect("solver should not be null");
        match solver.solve() {
            Some(solution) => {
                // Mind that `c_solution` must outlive `f` call!
                let c_solution = CString::new(solution).expect("solution should be valid cstring");
                f(c_solution.as_ptr());
            }
            None => f(ptr::null()),
        }
    }
}

#[no_mangle]
pub extern "C" fn solver_destroy(solver: *mut Solver) {
    assert!(!solver.is_null());
    unsafe {
        let _ = Box::from_raw(solver);
    }
}

#[no_mangle]
pub extern "C" fn http_deploy() {
    use http;
    http::deploy();
}
