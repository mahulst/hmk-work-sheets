#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use events::models::Event;
use db_parser::TimeRowEvent;

pub struct Row {
    pub person: u32,
    pub action: u32,
    pub minutes: u32,
}

pub struct Sheet {
    pub rows: Vec<Row>
}

pub struct WorkSheet {
    pub sheet: Sheet,
    pub actions: Vec<String>,
    pub employees: Vec<String>,
}

pub fn derive_work_sheet(entries: Vec<Event>) -> WorkSheet {
    entries
        .iter()
        .for_each(|event| {
            let time_entry: TimeRowEvent = serde_json::from_str(&event.payload).unwrap();

            // TODO: figure out how to create sheet from events
        });

    WorkSheet {
        sheet: Sheet { rows: vec![]},
        actions,
        employees
    }
}

