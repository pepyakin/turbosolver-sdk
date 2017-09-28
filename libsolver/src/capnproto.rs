
use std::io::BufReader;
use std::os::raw::c_void;
use std::sync::Arc;
use context::Context;
use capnp::serialize_packed;
use capnp::message::ReaderOptions;
use futures_cpupool::CpuPool;
use futures::future::*;
use self::errors::*;

mod errors {
    error_chain!{
        links {
            Context(::context::Error, ::context::ErrorKind);
        }
        foreign_links {
            Capnp(::capnp::Error);
        }
    }
}

enum Req {
    CreateSolver {
        grid: String,
    },
    Solve {
        id: usize,
    },
    Destroy {
        id: usize,
    },
}

impl Req {
    fn from_bytes(bytes: &[u8]) -> Result<Req> {
        use api_capnp::req::Which::*;

        let mut reader = BufReader::new(&bytes[..]);
        let msg = serialize_packed::read_message(&mut reader, ReaderOptions::new())?;
        let req_root = msg.get_root::<::api_capnp::req::Reader>()?;

        let req = match req_root.which() {
            Ok(CreateSolverReq(req)) => {
                let grid = req?.get_grid()?.to_string();
                Req::CreateSolver {
                    grid
                }
            },
            Ok(SolveReq(req)) => {
                let id = req?.get_id() as usize;
                Req::Solve {
                    id
                }
            },
            Ok(DestroyReq(req)) => {
                let id = req?.get_id() as usize;
                Req::Destroy {
                    id
                }
            },
            _ => panic!("unsupported variant. Is schema up to date?")
        };

        Ok(req)
    }
}

enum Resp {
    SolverCreated {
        id: usize,
    },
    SolverResult {
        solution: Option<String>
    },
    Destroyed,
}

impl Resp {
    fn to_bytes(self) -> Result<Vec<u8>> {
        let mut message = ::capnp::message::Builder::new_default();
        let req_builder = message.init_root::<::api_capnp::req::Builder>();

        match self {
            Resp::SolverCreated { id } => {
            },
            Resp::SolverResult { solution } => {

            },
            Resp::Destroyed => {
            },
        }

        panic!()
    }
}

struct Dispatcher {
    recv: extern "C" fn(),
    pool: CpuPool,
    ctx: Arc<Context>,
}

fn dispatch_msg(msg: Vec<u8>, ctx: Arc<Context>) -> Result<Vec<u8>> {
    let req = Req::from_bytes(&msg)?;
    let resp = match req {
        Req::CreateSolver { grid } => {
            let id = ctx.new_solver(&grid)?;
            Resp::SolverCreated { id }
        },
        Req::Solve { id } => {
            let solution = ctx.solve(id).ok();
            Resp::SolverResult { solution }
        },
        Req::Destroy { id } => {
            ctx.destroy(id)?;
            Resp::Destroyed
        }
    };

    Ok(resp.to_bytes()?)
}

impl Dispatcher {
    fn new(recv: extern "C" fn()) -> Dispatcher {
        Dispatcher {
            recv,
            pool: CpuPool::new(1),
            ctx: Arc::new(Context::new()),
        }
    }

    fn dispatch(&mut self, msg: Vec<u8>) {
        let ctx = Arc::clone(&self.ctx);
        let future = self.pool.spawn(lazy(move || {
            result(dispatch_msg(msg, ctx))
        }));
    }
}

#[no_mangle]
pub extern "C" fn capnp_init(recv: extern "C" fn()) -> *mut c_void {
    let dispatcher = Box::new(Dispatcher::new(recv));
    Box::into_raw(dispatcher) as *mut c_void
}

#[no_mangle]
pub extern "C" fn capnp_send(this: *mut c_void, msg: *const u8, msg_len: usize) {
    use std::slice;
    unsafe {
        let this = this as *mut Dispatcher;
        let dispatcher = this.as_mut().expect("this shouldn't be null");
        let msg = slice::from_raw_parts(msg, msg_len).to_vec();
        dispatcher.dispatch(msg);
    }
}
