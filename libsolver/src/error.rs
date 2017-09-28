
error_chain! {
    links {
        Context(::context::Error, ::context::ErrorKind);
    }
    foreign_links {
        Io(::std::io::Error);
        Utf8(::std::str::Utf8Error);
        Nul(::std::ffi::NulError);
        Parse(::solver::ParseError);
        Capnp(::capnp::Error);
    }
}
