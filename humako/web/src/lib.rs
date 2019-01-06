extern crate iron;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;


use iron::prelude::*;

fn hello_world(_: &mut Request) -> IronResult<Response> {
    let conn= events::establish_connection();
    let events = events::get_events(&conn);

    eprintln!("handler running");

    let json = serde_json::to_string(&events).unwrap();
    Ok(Response::with((iron::status::Ok, json)))
}

pub fn serve() {
    let mut chain = Chain::new(hello_world);
    Iron::new(chain).http("localhost:3000");
}
