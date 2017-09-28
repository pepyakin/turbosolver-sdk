//! Executor - actor-like processor of the incoming messages.

use std::sync::mpsc::{channel, Sender};
use std::thread;
use context::Context;
use error::*;

fn handle_req(req: Req, ctx: &mut Context) -> Resp {
    fn handle(req_kind: ReqKind, ctx: &mut Context) -> Result<RespKind> {
        let resp_kind = match req_kind {
            ReqKind::CreateSolver { grid } => {
                let id = ctx.new_solver(&grid)?;
                RespKind::SolverCreated { id }
            }
            ReqKind::Solve { id } => {
                let solution = ctx.solve(id)?;
                RespKind::SolverResult { solution }
            }
            ReqKind::Destroy { id } => {
                ctx.destroy(id)?;
                RespKind::Destroyed
            }
        };
        Ok(resp_kind)
    }

    let resp_kind = handle(req.kind, ctx);
    Resp {
        id: req.id,
        kind: resp_kind,
    }
}

pub trait ExecutorCallback: Send {
    fn call(&mut self, r: Resp);
}

impl<F: FnMut(Resp) + Send> ExecutorCallback for F {
    fn call(&mut self, resp: Resp) {
        self(resp)
    }
}

pub struct Executor {
    sender: Sender<Req>,
}

impl Executor {
    pub fn new<F: ExecutorCallback + 'static>(recv: F) -> Executor {
        let (tx, rx) = channel();
        let _ = thread::spawn(move || {
            let mut recv = recv;
            let mut ctx = Context::new();

            for req in rx {
                let resp = handle_req(req, &mut ctx);
                recv.call(resp);
            }
        });
        Executor { sender: tx }
    }

    pub fn send(&self, req: Req) {
        self.sender.send(req).unwrap();
    }
}

pub struct Req {
    pub id: usize,
    pub kind: ReqKind,
}

pub enum ReqKind {
    CreateSolver { grid: String },
    Solve { id: usize },
    Destroy { id: usize },
}

pub struct Resp {
    pub id: usize,
    pub kind: Result<RespKind>,
}

pub enum RespKind {
    SolverCreated { id: usize },
    SolverResult { solution: String },
    Destroyed,
}
