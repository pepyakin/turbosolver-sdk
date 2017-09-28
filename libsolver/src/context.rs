use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;
use std::mem;
use solver::Solver;

error_chain!{
    errors {
        BadGrid {
            description("grid couldn't be parsed")
        }
        NotAvailable(id: usize) {
            description("solver with specified id not available at the moment or doesn't exist")
        }
        SolutionNotFound {
            description("solution for the specified grid couldn't be found")
        }
    }
}

pub struct Context {
    next_id: AtomicUsize,
    solvers: Mutex<HashMap<usize, Solver>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            next_id: AtomicUsize::new(0),
            solvers: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_solver(&self, grid: &str) -> Result<usize> {
        let new_solver = Solver::from_str(&grid).chain_err(
            || ErrorKind::BadGrid,
        )?;
        let solver_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut solvers = self.solvers.lock().unwrap();
        solvers.insert(solver_id, new_solver);
        Ok(solver_id)
    }

    pub fn solve(&self, id: usize) -> Result<String> {
        let mut solver = {
            let mut solvers = self.solvers.lock().unwrap();
            match solvers.remove(&id) {
                Some(solver) => solver,
                None => bail!(ErrorKind::NotAvailable(id)),
            }
        };

        let maybe_solution = solver.solve();

        self.solvers.lock().unwrap().insert(id, solver);

        if let Some(solution) = maybe_solution {
            Ok(solution)
        } else {
            bail!(ErrorKind::SolutionNotFound);
        }
    }

    pub fn destroy(&self, id: usize) -> Result<()> {
        let mut solvers = self.solvers.lock().unwrap();
        match solvers.remove(&id) {
            Some(solver) => {
                mem::drop(solver);
                Ok(())
            }
            None => bail!(ErrorKind::NotAvailable(id)),
        }
    }
}
