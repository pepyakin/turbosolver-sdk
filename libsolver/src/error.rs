
error_chain! {
    foreign_links {
        Utf8(::std::str::Utf8Error);
        Nul(::std::ffi::NulError);
        Parse(::solver::ParseError);
    }
    errors {
        SolutionNotFound {
            description("solution for specified grid cannot be found")
        }
    }
}
