@0xe7f29aeaaa98259c;

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
        createSolverResp @1 :CreateSolverResp;
        solveResp @2 :SolveResp;
        destroyResp @3 :Void;
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

