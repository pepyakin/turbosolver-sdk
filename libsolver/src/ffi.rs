use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr;

use solver::*;

#[no_mangle]
pub extern "C" fn solver_create(sudoku_grid: *const c_char) -> *mut Solver {
    assert!(!sudoku_grid.is_null(), "sudoku_grid should not be null");
    unsafe {
        let cstr = CStr::from_ptr(sudoku_grid).to_str().unwrap();
        let solver = Box::new(Solver::from_str(cstr).unwrap());

        Box::into_raw(solver)
    }
}

#[no_mangle]
pub extern "C" fn solver_solve(solver: *mut Solver, f: fn(*const c_char)) {
    unsafe {
        let solver = solver.as_mut().expect("solver should not be null");
        match solver.solve() {
            Some(solution) => {
                let c_solution = CString::new(solution).expect("solution should be valid cstring");
                f(c_solution.as_ptr())
            }
            None => f(ptr::null()),
        }
    }
}

pub extern "C" fn solver_destroy(solver: *mut Solver) {
    assert!(!solver.is_null());
    unsafe {
        let _ = Box::from_raw(solver);
    }
}
