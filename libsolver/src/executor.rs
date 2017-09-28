//! Executor - actor-like processor of the incoming messages.

use std::sync::mpsc::{channel, Sender};
use std::thread;
use context::Context;
use context::*;

fn handle_req(req: Req, ctx: &mut Context) -> Result<Resp> {
    let resp_kind = match req.kind {
        ReqKind::CreateSolver { grid } => {
            let id = ctx.new_solver(&grid)?;
            RespKind::SolverCreated { id }
        }
        ReqKind::Solve { id } => {
            let solution = ctx.solve(id).ok();
            RespKind::SolverResult { solution }
        }
        ReqKind::Destroy { id } => {
            ctx.destroy(id)?;
            RespKind::Destroyed
        }
    };

    Ok(Resp {
        id: req.id,
        kind: resp_kind,
    })
}

pub struct Executor {
    sender: Sender<Req>,
}

impl Executor {
    pub fn new<F: FnMut(Result<Resp>) + Send + 'static>(recv: F) -> Executor {
        let (tx, rx) = channel();
        let _ = thread::spawn(move || {
            let mut recv = recv;
            let mut ctx = Context::new();

            for req in rx {
                let resp = handle_req(req, &mut ctx);
                recv(resp)
            }
        });
        Executor { sender: tx }
    }

    pub fn send(&self, req: Req) {
        self.sender.send(req).unwrap();
    }
}

#[derive(Clone)]
pub struct Req {
    pub id: usize,
    pub kind: ReqKind,
}

#[derive(Clone)]
pub enum ReqKind {
    CreateSolver { grid: String },
    Solve { id: usize },
    Destroy { id: usize },
}

#[derive(Clone)]
pub struct Resp {
    pub id: usize,
    pub kind: RespKind,
}

#[derive(Clone)]
pub enum RespKind {
    SolverCreated { id: usize },
    SolverResult { solution: Option<String> },
    Destroyed,
}
