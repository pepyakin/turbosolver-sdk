@0xe7f29aeaaa98259c;

struct Req {
    union {
        createSolverReq @0 :CreateSolverReq;
        solveReq @1 :SolveReq;
        destroyReq @2 :DestroyReq;
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
