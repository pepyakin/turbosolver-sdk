
use std::io::BufReader;
use std::os::raw::c_void;
use capnp::serialize;
use capnp::message::ReaderOptions;
use executor::{Executor, Req, ReqKind, Resp, RespKind};
use error::*;
use api_capnp;

impl Req {
    fn from_bytes(bytes: &[u8]) -> Result<Req> {
        use api_capnp::req::Which::*;

        let mut reader = BufReader::new(&bytes[..]);
        let msg = serialize::read_message(&mut reader, ReaderOptions::new())?;
        let req_root = msg.get_root::<api_capnp::req::Reader>()?;

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
    fn to_bytes(self) -> Vec<u8> {
        let mut message = ::capnp::message::Builder::new_default();
        {
            let mut resp_builder = message.init_root::<api_capnp::resp::Builder>();
            resp_builder.set_id(self.id as u32);

            match self.kind {
                Ok(kind) => {
                    let mut ok_resp = resp_builder.borrow().init_ok();
                    match kind {
                        RespKind::SolverCreated { id } => {
                            let mut resp = ok_resp.borrow().init_create_solver_resp();
                            resp.set_id(id as u32);
                        }
                        RespKind::SolverResult { ref solution } => {
                            let mut resp = ok_resp.borrow().init_solve_resp();
                            resp.set_solution(solution);
                        }
                        RespKind::Destroyed => {
                            ok_resp.borrow().set_destroy_resp(());
                        }
                    }
                }
                Err(_e) => {
                    // TODO: turn _e into errno.
                    resp_builder.set_err(1);
                }
            }
        }

        let mut bytes = Vec::new();
        serialize::write_message(&mut bytes, &message).expect("write_message should succeed");
        bytes
    }
}

#[no_mangle]
pub extern "C" fn capnp_init(recv: extern "C" fn(*const u8, usize)) -> *mut c_void {
    let f = move |resp: Resp| {
        let bytes = resp.to_bytes();
        recv(bytes.as_ptr(), bytes.len())
    };
    let dispatcher = Box::new(Executor::new(f));
    Box::into_raw(dispatcher) as *mut c_void
}

#[no_mangle]
pub extern "C" fn capnp_send(this: *mut c_void, msg: *const u8, msg_len: usize) {
    use std::slice;
    unsafe {
        let this = this as *mut Executor;
        let executor = this.as_mut().expect("`this` shouldn't be null");
        let msg = slice::from_raw_parts(msg, msg_len);
        let req = Req::from_bytes(&msg).expect("wrong schema?");
        executor.send(req);
    }
}

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod jni {
    extern crate jni;

    use super::*;
    use executor::ExecutorCallback;

    use self::jni::JNIEnv;
    use self::jni::objects::{GlobalRef, JClass, JObject, JValue};
    use self::jni::sys::jlong;
    use self::jni::sys::JNIEnv as RawJNIEnv;
    use self::jni::sys::{JavaVM, jbyteArray};

    use std::ptr;
    use std::os::raw::c_void;

    struct Context {
        vm: *mut JavaVM,
        dispatcher_this: GlobalRef,
    }

    impl ExecutorCallback for Context {
        fn call(&mut self, r: Resp) {
            let mut result_bytes = r.to_bytes();

            with_attached_thread(self.vm, |env| {
                let byte_buf = env.new_direct_byte_buffer(&mut result_bytes).unwrap();
                let byte_buf_obj = JValue::Object(*byte_buf);

                let dispatcher_this = self.dispatcher_this.as_obj();
                env.call_method(
                    dispatcher_this,
                    "callback",
                    "(Ljava/nio/ByteBuffer;)V",
                    &[byte_buf_obj],
                ).unwrap();
            });
        }
    }

    // It should be safe provided that we use JVM only via
    // attached to the current thread JNIEnv.
    unsafe impl Send for Context {}

    /// Attach JavaVM to the current thread, do the work and then detach.
    fn with_attached_thread<F: FnOnce(JNIEnv)>(vm: *mut JavaVM, f: F) {
        unsafe {
            let attach_current_thread = (**vm).AttachCurrentThread.expect(
                "JavaVM doesn't have AttachCurrentThread",
            );
            let detach_current_thread = (**vm).DetachCurrentThread.expect(
                "JavaVM doesn't have DetachCurrentThread",
            );

            // Attach JavaVM to the current thread. This would provide
            // JNIEnv for further use.
            let mut raw_env: *mut RawJNIEnv = ptr::null_mut();
            let result = attach_current_thread(
                vm,
                &mut raw_env as *mut *mut _ as *mut *mut c_void,
                ptr::null_mut(),
            );
            assert!(result == 0);

            let env = JNIEnv::from_raw(raw_env).expect("from_raw failed (raw_env is null?)");
            f(env);

            // Detach JavaVM from the current thread after we done with it.
            let result = detach_current_thread(vm);
            assert!(result == 0);
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_me_pepyakin_turbosolver_capnp_Dispatcher_capnp_1init(
        env: JNIEnv,
        _: JClass,
        this: JObject,
    ) -> jlong {
        // Pin `this` object, this will prevent `this` to be garbage collected.
        let dispatcher_this = env.new_global_ref(this).unwrap();

        unsafe {
            // Extract JavaVM: it will be needed later in the callback.
            let raw_env = env.get_native_interface();
            let mut java_vm: *mut JavaVM = ptr::null_mut();
            let get_java_vm = (**raw_env).GetJavaVM.unwrap();
            let result = get_java_vm(raw_env, &mut java_vm as *mut *mut _);
            assert!(result == 0);

            let ctx = Context {
                vm: java_vm,
                dispatcher_this,
            };
            let executor = Box::new(Executor::new(ctx));
            Box::into_raw(executor) as *mut c_void as jlong
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_me_pepyakin_turbosolver_capnp_Dispatcher_capnp_1send(
        env: JNIEnv,
        _: JClass,
        executor_ptr: jlong,
        data: jbyteArray,
    ) {
        // TODO: Can we get away without copying here?
        let data_vec = env.convert_byte_array(data).expect("couldn't copy `data`");
        let req = Req::from_bytes(&data_vec).expect("wrong schema?");

        unsafe {
            let executor = executor_ptr as *mut c_void as *mut Executor;
            executor.as_mut().expect("`executor` is null").send(req);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let resp = Resp {
            id: 228,
            kind: Ok(RespKind::SolverResult {
                solution: "hello world".to_string(),
            }),
        };
        let _bytes = resp.to_bytes();
        // TODO: assert_eq
    }
}
