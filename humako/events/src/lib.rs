#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models;

use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

use self::schema::events;
use self::models::*;
use self::diesel::prelude::*;
use db_parser::TimeRowEvent;

pub fn print_events() {
    use self::schema::events::dsl::*;

    let connection = establish_connection();
    let results = events
        .load::<Event>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} events", results.len());
    for event in results {
        println!("----------\n");
        println!("{:?}", event);
        println!("----------\n");
    }
}

pub fn get_events(conn: &PgConnection) -> Vec<Event> {
    use self::schema::events::dsl::*;

    let results = events
        .load::<Event>(conn)
        .expect("Error loading events");

    results
}

pub fn save_event(conn: &PgConnection, event: TimeRowEvent) {
    save_events(conn, vec![event]);
}

pub fn save_events(conn: &PgConnection, list_of_events: Vec<TimeRowEvent>) {
    let mut result = vec![];

    list_of_events.iter().for_each(|event| {
        let my_uuid = uuid::Uuid::new_v4();
        let payload: String = event.stringify();

        let event = NewEvent {
            id: my_uuid,
            unique_id: event.id as i32,
            event_type: "time_row_event",
            payload: payload,
        };
        result.push(event);
    });

    diesel::insert_into(events::table)
        .values(&result)
        .on_conflict(events::unique_id)
        .do_update()
        .set(events::payload.eq(diesel::pg::upsert::excluded(events::payload)))
        .execute(conn)
        .expect("insert failed");
}