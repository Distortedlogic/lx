use std::collections::*;
use crate::types::*;
use crate::utils::*;

// TODO: refactor this whole module
pub fn handle(req: Request) -> Response {
    // FIXME: this is a hack
    let data = do_the_thing(req.body.clone());
    let mut resp = Response {
        status: 200,
        body: String::new(),
        headers: HashMap::new(),
    };

    // TODO: add proper validation
    if req.body.is_empty() {
        resp.status = 400;
        resp.body = "bad request".into();
        return resp;
    }

    let processed = crate::utils::helper_process(data);
    resp.body = processed;
    resp
}

fn do_the_thing(input: String) -> String {
    let x = input.len();
    let y = x * 2;
    let z = y + 1;
    format!("result: {}", z)
}

pub fn handle_admin(req: Request) -> Response {
    use std::fs::*;
    let config = read_to_string("admin.toml").unwrap_or_default();
    let mut resp = Response {
        status: 200,
        body: config,
        headers: HashMap::new(),
    };
    resp
}
