extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;


extern crate router;
extern crate iron;
extern crate multipart;

use std::io::{self, Write};
use multipart::mock::StdoutTee;
use multipart::server::{Multipart, Entries, SaveResult};
use multipart::server::save::SavedData;
use iron::prelude::*;
use iron::status;
use tempfile::tempdir;
use std::collections::HashMap;
use iron_cors::CorsMiddleware;

use iron::prelude::*;
use iron::Handler;

pub fn serve() {
    let mut router = router::Router::new();

    router.route(iron::method::Get, "hello", |_: &mut Request| {
        Ok(Response::with((iron::status::Ok, "Hello world !")))
    }, "hello");

    router.route(iron::method::Post, "upload", process_request, "hello2");
    let cors_middleware = CorsMiddleware::with_allow_any();
    let mut chain = Chain::new(router);
    chain.link_around(cors_middleware);

    Iron::new(chain).http("localhost:3010");
}

fn process_request(request: &mut Request) -> IronResult<Response> {
    // Getting a multipart reader wrapper
    match Multipart::from_request(request) {
        Ok(mut multipart) => {
            // Fetching all data and processing it.
            // save().temp() reads the request fully, parsing all fields and saving all files
            // in a new temporary directory under the OS temporary directory.
            match multipart.save().temp() {
                SaveResult::Full(entries) => process_entries(entries),
                SaveResult::Partial(entries, reason) => {
                    process_entries(entries.keep_partial())?;
                    Ok(Response::with((
                        status::BadRequest,
                        format!("error reading request: {}", reason.unwrap_err())
                    )))
                }
                SaveResult::Error(error) => Ok(Response::with((
                    status::BadRequest,
                    format!("error reading request: {}", error)
                ))),
            }
        }
        Err(_) => {
            Ok(Response::with((status::BadRequest, "The request is not multipart")))
        }
    }
}

fn process_entries(entries: Entries) -> IronResult<Response> {
    let field = entries.fields
        .get(&"file".to_string())
        .expect("Please upload file under key \"file\"")
        .first()
        .expect("Please upload file under key \"file\"");

    match &field.data {
        SavedData::File(path, _) => {
            let time_events = db_parser::parse_db(path);

            let conn = events::establish_connection();
            events::save_events(&conn, time_events);
            let events = events::get_events(&conn);

            let worksheet = worksheets::derive_work_sheet(events);

            Ok(Response::with((status::Ok, serde_json::to_string(&worksheet).unwrap())))
        }
        _ => {
            Ok(Response::with((status::BadRequest, "Nope")))
        }
    }
}