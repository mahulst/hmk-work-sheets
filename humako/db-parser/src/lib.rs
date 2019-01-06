extern crate csv;
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::process::Command;
use stringreader::StringReader;
use std::collections::HashMap;
use chrono::NaiveDateTime;
use serde::ser::{Serialize, Serializer, SerializeStruct};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct TimeEntryRaw {
    Date: String,
    Time: String,
    Empl: u32,
    Action: u32,
    TRD_RunNr: u32,
}


#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct ActionsRaw {
    ACT_ID: u32,
    ACT_Name: String,
}


#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct EmployeeRaw {
    EN: u32,
    Name: String,
}

pub type Employees = HashMap<u32, String>;
pub type Actions = HashMap<u32, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRowEvent<'a> {
    pub id: u32,
    pub employee: &'a str,
    pub action: &'a str,
    pub timestamp: NaiveDateTime,
}

impl<'a> TimeRowEvent<'a>  {
    pub fn stringify(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}


pub fn parse_db() {
    let time = read_time_entries();
    let employees = get_employees();
    let actions = get_actions();
    let default_value = &String::from("Onbekend");

    time
        .into_iter()
        .map(|time_entry| {
            let employee = employees
                .get(&time_entry.Empl)
                .unwrap_or(default_value);

            let action = actions
                .get(&time_entry.Action)
                .unwrap_or(default_value);

            let date = chrono::NaiveDate::parse_from_str(&time_entry.Date, "%Y%m%d")
                .expect("Invalid date");

            let time = chrono::NaiveTime::parse_from_str(&time_entry.Time, "%k%M%S")
                .expect("Invalid time");

            let timestamp = NaiveDateTime::new(date, time);

            TimeRowEvent {
                id: time_entry.TRD_RunNr,
                employee,
                action,
                timestamp,
            }
        })
        .for_each(|time| {
            eprintln!("time = {:#?}", time);
        });
}

fn read_time_entries() -> Vec<TimeEntryRaw> {
    let result = Command::new("mdb-export")
        .arg("/Users/michelvanderhulst/projects/rust/humako/humako/db-parser/GreenSpy.mdb")
        .arg("Time_RawData")
        .output()
        .expect("get csv of data").stdout;
    let csv = std::str::from_utf8(&result).expect("Can't stringify stdout");
    let streader = StringReader::new(csv);

    let mut rdr = csv::Reader::from_reader(streader);

    rdr.deserialize()
        .into_iter()
        .map(|record| {
            let mut time_entry: TimeEntryRaw = record.expect("Invalid TimeEntryRaw");
            if time_entry.Time.len() == 5 {
                let mut time = String::from("0");
                time.push_str(&time_entry.Time);
                time_entry.Time = time;
            }
            time_entry
        })
        .collect()
}

pub fn get_employees() -> Employees {
    let result = Command::new("mdb-export")
        .arg("/Users/michelvanderhulst/projects/rust/humako/humako/db-parser/GreenSpy.mdb")
        .arg("PersonelData")
        .output()
        .expect("get csv of data").stdout;
    let csv = std::str::from_utf8(&result).expect("Can't stringify stdout");
    let streader = StringReader::new(csv);

    csv::Reader::from_reader(streader)
        .deserialize()
        .into_iter()
        .map(|record| record.expect("Invalid EmployeeRaw"))
        .fold(HashMap::new(), |mut map, record: EmployeeRaw| {
            map.insert(record.EN, record.Name);

            map
        })
}

pub fn get_actions() -> Actions {
    let result = Command::new("mdb-export")
        .arg("/Users/michelvanderhulst/projects/rust/humako/humako/db-parser/GreenSpy.mdb")
        .arg("Actions")
        .output()
        .expect("get csv of data").stdout;
    let csv = std::str::from_utf8(&result).expect("Can't stringify stdout");
    let streader = StringReader::new(csv);

    csv::Reader::from_reader(streader)
        .deserialize()
        .into_iter()
        .map(|record| record.expect("Invalid ActionRaw"))
        .fold(HashMap::new(), |mut map, record: ActionsRaw| {
            map.insert(record.ACT_ID, record.ACT_Name);

            map
        })
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
