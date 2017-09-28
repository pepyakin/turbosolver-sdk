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
        err @1 :UInt32;
        ok @2 :OkResp;
    }
}

struct OkResp {
    union {
        createSolverResp @0 :CreateSolverResp;
        solveResp @1 :SolveResp;
        destroyResp @2 :Void;
    }
}

struct CreateSolverReq {
    grid @0 :Text;
}

struct SolveReq {
    id @0 :UInt32;
}

struct DestroyReq {
    id @0 :UInt32;
}

struct CreateSolverResp {
    id @0 :UInt32;
}

struct SolveResp {
    solution @0 :Text;
}
