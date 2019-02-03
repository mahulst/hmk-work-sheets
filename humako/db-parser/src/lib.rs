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
use std::path::PathBuf;

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
pub struct TimeRowEvent {
    pub id: u32,
    pub employee: String,
    pub action: String,
    pub timestamp: NaiveDateTime,
}

impl TimeRowEvent {
    pub fn stringify(&self) -> String {
        serde_json::to_string(self).expect("stringify failed")
    }
}


pub fn parse_db(path_to_db: &PathBuf) -> Vec<TimeRowEvent> {
    let time = read_time_entries(path_to_db);
    let employees = get_employees(path_to_db);
    let actions = get_actions(path_to_db);
    let default_value = &String::from("Onbekend");

    time
        .into_iter()
        .map(|time_entry| {
            let employee = employees
                .get(&time_entry.Empl)
                .unwrap_or(default_value)
                .clone();

            let action = actions
                .get(&time_entry.Action)
                .unwrap_or(default_value)
                .clone();

            let date = chrono::NaiveDate::parse_from_str(&time_entry.Date, "%Y%m%d")
                .unwrap_or(chrono::NaiveDate::from_ymd(1970, 1, 1));

            let time = chrono::NaiveTime::parse_from_str(&time_entry.Time, "%k%M%S")
                .unwrap_or(chrono::NaiveTime::from_hms(0, 0, 0));

            let timestamp = NaiveDateTime::new(date, time);

            TimeRowEvent {
                id: time_entry.TRD_RunNr,
                employee,
                action,
                timestamp,
            }
        })
        .collect()
}

fn read_time_entries(path_to_db: &PathBuf) -> Vec<TimeEntryRaw> {
    let result = Command::new("mdb-export")
        .arg(path_to_db)
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

pub fn get_employees(path_to_db: &PathBuf) -> Employees {
    let result = Command::new("mdb-export")
        .arg(path_to_db)
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

pub fn get_actions(path_to_db: &PathBuf) -> Actions {
    let result = Command::new("mdb-export")
        .arg(path_to_db)
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
