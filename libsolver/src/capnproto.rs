
use std::io::BufReader;
use std::os::raw::c_void;
use capnp::serialize;
use capnp::message::ReaderOptions;
use self::errors::*;
use executor::{Executor, Req, ReqKind, Resp, RespKind};

mod errors {
    error_chain!{
        links {
            Context(::context::Error, ::context::ErrorKind);
        }
        foreign_links {
            Io(::std::io::Error);
            Capnp(::capnp::Error);
        }
    }
}

impl Req {
    fn from_bytes(bytes: &[u8]) -> Result<Req> {
        use api_capnp::req::Which::*;

        let mut reader = BufReader::new(&bytes[..]);
        let msg = serialize::read_message(&mut reader, ReaderOptions::new())?;
        let req_root = msg.get_root::<::api_capnp::req::Reader>()?;

        let kind = match req_root.which() {
            Ok(CreateSolverReq(req)) => {
                let grid = req?.get_grid()?.to_string();
                ReqKind::CreateSolver { grid }
            }
            Ok(SolveReq(req)) => {
                let id = req?.get_id() as usize;
                ReqKind::Solve { id }
            }
            Ok(DestroyReq(req)) => {
                let id = req?.get_id() as usize;
                ReqKind::Destroy { id }
            }
            _ => panic!("unsupported variant. Is schema up to date?"),
        };

        let id = req_root.get_id() as usize;

        Ok(Req { id, kind })
    }
}

impl Resp {
    fn to_bytes(self) -> Result<Vec<u8>> {
        let mut message = ::capnp::message::Builder::new_default();
        {
            let mut resp_builder = message.init_root::<::api_capnp::resp::Builder>();
            resp_builder.set_id(self.id as u32);

            match self.kind {
                RespKind::SolverCreated { id } => {
                    let mut resp = resp_builder.borrow().init_create_solver_resp();
                    resp.set_id(id as u32);
                }
                RespKind::SolverResult { solution } => {
                    let mut resp = resp_builder.borrow().init_solve_resp();
                    if let Some(ref solution) = solution {
                        resp.set_solution(solution);
                    }
                }
                RespKind::Destroyed => {
                    resp_builder.borrow().set_destroy_resp(());
                }
            }
        }

        let mut bytes = Vec::new();
        serialize::write_message(&mut bytes, &message)?;

        Ok(bytes)
    }
}

struct Dispatcher {
    recv: extern "C" fn(*const u8, usize),
    executor: Executor,
}

impl Dispatcher {
    fn new(recv: extern "C" fn(*const u8, usize)) -> Dispatcher {
        let f = move |resp: ::context::Result<Resp>| {
            let bytes = resp.unwrap().to_bytes().unwrap();
            recv(bytes.as_ptr(), bytes.len())
        };

        Dispatcher {
            recv,
            executor: Executor::new(f),
        }
    }

    fn dispatch(&mut self, msg: Vec<u8>) {
        let req = Req::from_bytes(&msg).expect("wrong schema?");
        self.executor.send(req);
    }
}

#[no_mangle]
pub extern "C" fn capnp_init(recv: extern "C" fn(*const u8, usize)) -> *mut c_void {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let resp = Resp {
            id: 228,
            kind: RespKind::SolverResult { solution: Some("hello world".to_string()) },
        };
        let _bytes = resp.to_bytes().unwrap();
        // TODO: assert_eq
    }
}
