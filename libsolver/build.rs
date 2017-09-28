extern crate capnpc;

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("../common")
        .file("../common/api.capnp")
        .run()
        .expect("schema compiler command");
}
