#![allow(proc_macro_derive_resolution_fallback)]
extern crate diesel;
extern crate serde;
extern crate serde_json;

extern crate uuid;

use super::schema::events;

#[derive(Queryable, Debug, Serialize)]
pub struct Event {
    pub id: uuid::Uuid,
    pub unique_id: i32,
    pub event_type: String,
    pub payload: String,
    pub timestamp: chrono::NaiveDateTime,
}


#[derive(Insertable, Debug)]
#[table_name = "events"]
pub struct NewEvent<'a> {
    pub id: uuid::Uuid,
    pub unique_id: i32,
    pub event_type: &'a str,
    pub payload: String,
}