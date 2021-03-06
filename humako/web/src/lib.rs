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
use std::collections::HashMap;
use iron_cors::CorsMiddleware;
use std::env;
use iron::prelude::*;
use iron::Handler;

pub fn serve() {
    let mut router = router::Router::new();

    router.route(iron::method::Get, "/hello", |_: &mut Request| {
        Ok(Response::with((iron::status::Ok, "Hello world !")))
    }, "hello");

    router.route(iron::method::Get, "/day/:date", get_work_sheet, "get_events");
    router.route(iron::method::Get, "/available-days", get_available_days, "get_available_days");

    router.route(iron::method::Post, "/upload", process_request, "hello2");
    let cors_middleware = CorsMiddleware::with_allow_any();
    let mut chain = Chain::new(router);
    chain.link_around(cors_middleware);

    Iron::new(chain).http("0.0.0.0:3010");
}

fn get_available_days(request: &mut Request) -> IronResult<Response> {
    let conn = events::establish_connection();
    let events = events::get_events(&conn);

    let worksheet = worksheets::derive_work_sheet(events);

    let available_days: Vec<&chrono::NaiveDate> = worksheet.keys().collect();
    let json = serde_json::to_string(&available_days).unwrap();

    Ok(Response::with((status::Ok, json)))
}

fn get_work_sheet(request: &mut Request) -> IronResult<Response> {
    let params = request.extensions.get::<router::Router>().unwrap();
    match &params.find("date")
        .and_then(|d: &str| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()
        }) {
        Some(date) => {
            let conn = events::establish_connection();
            let events = events::get_events(&conn);

            let worksheet = worksheets::derive_work_sheet(events);


            let day = worksheet.get(&date);

            match day {
                Some(data) => {
                    Ok(Response::with((status::Ok, serde_json::to_string(&data).unwrap())))
                }
                None => Ok(Response::with((status::BadRequest, "Incorrect date submitted")))
            }
        }
        None => Ok(Response::with((status::BadRequest, "Incorrect date submitted")))
    }
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