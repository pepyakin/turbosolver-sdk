use rocket;
use rocket::{State, Response, Request};
use rocket::response::Responder;
use rocket::http::{Status, ContentType};
use rocket_contrib::{Json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::mem;
use solver::Solver;
use context::Context;

mod errors {
    error_chain! {
        links {
            Context(::context::Error, ::context::ErrorKind);
            Lib(::error::Error, ::error::ErrorKind);
        }
    }
}


#[derive(Deserialize)]
struct CreateSolverReq {
    grid: String,
}

#[post("/", data = "<req>")]
fn create(req: Json<CreateSolverReq>, ctx: State<Context>) -> Result<Json<Value>, errors::Error> {
    let solver_id = ctx.new_solver(&req.grid)?;
    let resp = Json(json!({
        "id": solver_id as u32
    }));
    Ok(resp)
}

#[get("/<id>/solution")]
fn solution(id: usize, ctx: State<Context>) -> Result<Json<Value>, errors::Error> {
    let solution = ctx.solve(id)?;
    let resp = Json(json!({
        "solution": solution
    }));
    Ok(resp)
}

#[delete("/<id>")]
fn delete(id: usize, ctx: State<Context>) -> Result<(), errors::Error> {
    Ok(ctx.destroy(id)?)
}

impl<'a> Responder<'a> for errors::Error {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        use std::io::Cursor;

        let description = self.description();
        let resp_body = json!({
                "description": description,
            }).to_string();

        let resp = Response::build()
            .status(Status::BadRequest)
            .header(ContentType::JSON)
            .sized_body(Cursor::new(resp_body))
            .finalize();
        Ok(resp)
    }
}

fn create_rocket() -> rocket::Rocket {
    let ctx = Context::new();
    rocket::ignite().manage(ctx).mount(
        "/",
        routes![
            create,
            solution,
            delete,
        ],
    )
}

/// Deploy and run http server instance.
///
/// Be aware! This will block the calling thread.
#[no_mangle]
pub extern "C" fn http_deploy() {
    let rocket = create_rocket();
    rocket.launch();
}

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod jni {
    extern crate jni;

    use super::*;
    use self::jni::JNIEnv;
    use self::jni::objects::JClass;

    #[no_mangle]
    pub extern "C" fn Java_me_pepyakin_turbosolver_LocalHttpTurboSolverFactory_deploy(
        env: JNIEnv,
        _: JClass,
    ) {
        http_deploy();
    }
}

#[cfg(test)]
mod tests {
    use rocket::local::{Client, LocalResponse};
    use rocket::http::{ContentType, Status};

    fn create_custom<'c>(client: &'c Client, grid: &str) -> LocalResponse<'c> {
        client
            .post("/")
            .header(ContentType::JSON)
            .body(
                json!({
                    "grid": grid
                }).to_string(),
            )
            .dispatch()
    }

    fn create(client: &Client) -> LocalResponse {
        let sudoku_grid = "\
___|2__|_63
3__|__5|4_1
__1|__3|98_
___|___|_9_
___|538|___
_3_|___|___
_26|3__|5__
5_3|7__|__8
47_|__1|___";
        create_custom(client, sudoku_grid)
    }

    fn solution(client: &Client, id: usize) -> LocalResponse {
        client.get(format!("/{}/solution", id)).dispatch()
    }

    fn delete(client: &Client, id: usize) -> LocalResponse {
        client.delete(format!("/{}", id)).dispatch()
    }

    #[test]
    fn test_create() {
        let rocket = super::create_rocket();
        let client = Client::new(rocket).unwrap();
        let mut response = create(&client);

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.body_string().unwrap(),
            json!({
                    "id": 0
                }).to_string()
        );
    }

    #[test]
    fn test_solution() {
        let rocket = super::create_rocket();
        let client = Client::new(rocket).unwrap();
        let _ = create(&client);
        let mut response = solution(&client, 0);

        let solution = "
854 219 763
397 865 421
261 473 985

785 126 394
649 538 172
132 947 856

926 384 517
513 792 648
478 651 239";

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.body_string().unwrap(),
            solution
        );
    }

    #[test]
    fn test_delete() {
        let rocket = super::create_rocket();
        let client = Client::new(rocket).unwrap();
        let _ = create(&client);
        let mut response = delete(&client, 0);

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.body_string(),
            None
        );
    }

    #[test]
    fn test_err_not_available() {
        let rocket = super::create_rocket();
        let client = Client::new(rocket).unwrap();
        let mut response = delete(&client, 0);

        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.body_string().unwrap(), json!({
                    "description": "solver with specified id not available \
                    at the moment or doesn't exist"
                }).to_string())
    }

    #[test]
    fn test_err_solution_not_found() {
        let sudoku_grid = "\
1__|___|___
_1_|___|___
__1|___|___
___|___|___
___|___|___
___|___|___
___|___|___
___|___|___
___|___|___";

        let rocket = super::create_rocket();
        let client = Client::new(rocket).unwrap();
        let _ = create_custom(&client, sudoku_grid);
        let mut response = solution(&client, 0);

        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.body_string().unwrap(), json!({
                    "description": "solution for the specified grid couldn't be found"
                }).to_string())
    }

    #[test]
    fn test_err_bad_grid() {
        let rocket = super::create_rocket();
        let client = Client::new(rocket).unwrap();
        let mut response = create_custom(&client, "<bad grid>");

        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.body_string().unwrap(), json!({
                    "description": "grid couldn't be parsed"
                }).to_string())
    }
}
