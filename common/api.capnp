@0xe7f29aeaaa98259c;

using Java = import "/capnp/java.capnp";
$Java.package("me.pepyakin.turbosolver.capnp");
$Java.outerClassname("Api");

struct Req {
    id @0 :UInt32;
    union {
        createSolverReq @1 :CreateSolverReq;
        solveReq @2 :SolveReq;
        destroyReq @3 :DestroyReq;
    }
}

struct Resp {
    id @0 :UInt32;
    union {
        errno @1 :UInt32;
        success @2 :SuccessfulResponse;
    }
}

struct SuccessfulResponse {
    union {
        createSolverResp @0 :CreateSolverResp;
        solveResp @1 :SolveResp;
        destroyResp @2 :Void;
    }
}

struct CreateSolverReq {
    grid @0 :Text;
}

struct CreateSolverResp {
    id @0 :UInt32;
}

struct SolveReq {
    id @0 :UInt32;
}

struct SolveResp {
    solution @0 :Text;
}

struct DestroyReq {
    id @0 :UInt32;
}

