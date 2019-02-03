extern crate serde_json;

extern crate serde_derive;

use std::collections::HashMap;
use events::models::Event;
use db_parser::TimeRowEvent;
use chrono::NaiveDate;
use chrono::NaiveDateTime;

pub fn derive_work_sheet(events: Vec<Event>) -> HashMap<NaiveDate, HashMap<String, HashMap<String, i32>>> {
    let mut days: HashMap<NaiveDate, HashMap<String, HashMap<String, i32>>> = HashMap::new();

    let mut time_entries_per_employee: HashMap<String, Vec<TimeRowEvent>> = events
        .iter()
        .map(|event| {
            let time_entry: TimeRowEvent = serde_json::from_str(&event.payload).expect("parsing failed");
            return time_entry;
        })
        .fold(HashMap::new(), |mut map, time_entry: TimeRowEvent| {
            let rows = map.entry(time_entry.employee.clone()).or_insert(vec![]);
            rows.push(time_entry);

            map
        });

    time_entries_per_employee
        .iter_mut()
        .for_each(|(_employee, entries)| {
            entries.sort_by(|a, b| {
                a.timestamp.cmp(&b.timestamp)
            })
        });

    let mut last_action: Option<String> = None;
    let mut last_date: Option<NaiveDateTime> = None;
    let mut last_break_date: Option<NaiveDateTime> = None;
    let mut break_time: i32 = 0;

    time_entries_per_employee
        .iter()
        .for_each(|(employee, time_entries)| {
            last_date = None;
            last_action = None;
            last_break_date = None;
            break_time = 0;

            time_entries
                .iter()
                .for_each(|time_row| {
                    match last_date {
                        Some(datetime) => {
                            if datetime.date() != time_row.timestamp.date() {
                                // New day
                                last_action = None;
                                last_break_date = None;
                            }

                            if time_row.action == "Begin/Pauze" {
                                match last_break_date {
                                    Some(break_date) => {
                                        // End of break
                                        let diff = time_row.timestamp.timestamp_millis() - break_date.timestamp_millis();
                                        let diff = diff / 1000 / 60;

                                        break_time += diff as i32;
                                        last_break_date = None;
                                    }
                                    None if last_action.is_some() => {
                                        // Start of break
                                        last_break_date = Some(time_row.timestamp.clone());
                                    }
                                    None => {
                                        // Start of new action in new day
                                        break_time = 0;
                                        last_date = Some(time_row.timestamp.clone());
                                        last_action = Some(time_row.action.clone());
                                    }
                                }
                            } else {
                                // End of action
                                let day = days
                                    .entry(datetime.date())
                                    .or_insert(HashMap::new());

                                let employee_map = day
                                    .entry(employee.clone())
                                    .or_insert(HashMap::new());

                                let action_to_mutate = employee_map
                                    .entry(time_row.action.clone())
                                    .or_insert(0);

                                match &last_action {
                                    Some(_action) => {
                                        let diff = time_row.timestamp.timestamp_millis() - datetime.timestamp_millis();
                                        let diff = diff / 1000 / 60;
                                        let diff_minus_break = diff - break_time as i64;
                                        *action_to_mutate += diff_minus_break as i32;
                                        break_time = 0;
                                        last_break_date = None;
                                    }
                                    _ => {
                                        // First action of the day was not "Begin/Pauze"
                                        // Should be fixed in source database
                                    }
                                }

                                last_date = Some(time_row.timestamp.clone());
                                last_action = Some(time_row.action.clone());
                            }
                        }
                        None => {
                            // Start of day
                            last_date = Some(time_row.timestamp.clone());
                            last_action = Some(time_row.action.clone());
                        }
                    }
                });
        });

    days
}


#[cfg(test)]
mod tests {
    use events::models::Event;
    use chrono::NaiveDateTime;
    use std::collections::HashMap;
    use chrono::NaiveDate;

    #[test]
    fn it_should_convert_events_to_work_sheet() {
        // Arrange
        let events: Vec<Event> = vec![
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 173,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":173,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T07:01:16\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 188,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":188,\"employee\":\"Michel\",\"action\":\"Kas\",\"timestamp\":\"2019-01-02T08:59:42\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 209,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":209,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T10:03:00\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 216,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":216,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T10:19:06\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 236,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":236,\"employee\":\"Michel\",\"action\":\"toppen B\",\"timestamp\":\"2019-01-02T10:25:22\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 264,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":264,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T12:50:44\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 264,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":264,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T13:19:44\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 272,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":272,\"employee\":\"Michel\",\"action\":\"steken B\",\"timestamp\":\"2019-01-02T13:22:30\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 285,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":285,\"employee\":\"Michel\",\"action\":\"stek plukken B\",\"timestamp\":\"2019-01-02T15:54:29\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
        ];
        let mut actions: HashMap<String, i32> = HashMap::new();
        actions.insert("Kas".to_string(), 118);
        actions.insert("toppen B".to_string(), 69);
        actions.insert("steken B".to_string(), 148);
        actions.insert("stek plukken B".to_string(), 151);

        // Act
        let work_sheet = crate::derive_work_sheet(events);
        let result = work_sheet
            .get(&NaiveDate::from_ymd(2019, 1, 2))
            .unwrap()
            .get("Michel")
            .unwrap();

        // Assert
        assert_eq!(result, &actions);
    }

    #[test]
    fn it_should2() {

        // Arrange
        let events: Vec<Event> = vec![
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 172,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":172,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T07:01:07\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 210,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":210,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T10:03:20\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 225,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":225,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T10:20:19\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 254,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":254,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T13:20:00\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 293,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":293,\"employee\":\"Michel\",\"action\":\"gewasverz opkw\",\"timestamp\":\"2019-01-02T16:09:17\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 294,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":294,\"employee\":\"Michel\",\"action\":\"opkweek divers B\",\"timestamp\":\"2019-01-02T16:18:22\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
            Event {
                id: uuid::Uuid::new_v4(),
                unique_id: 404,
                event_type: "time_row_event".to_string(),
                payload: "{\"id\":404,\"employee\":\"Michel\",\"action\":\"Begin/Pauze\",\"timestamp\":\"2019-01-02T12:54:21\"}".to_string(),
                timestamp: NaiveDateTime::parse_from_str("2019-01-11T15:33:43.390750", "%Y-%m-%dT%H:%M:%S%.f").unwrap(),
            },
        ];

        // Act
        let mut actions: HashMap<String, i32> = HashMap::new();
        actions.insert("gewasverz opkw".to_string(), 507);
        actions.insert("opkweek divers B".to_string(), 9);

        // Act
        let work_sheet = crate::derive_work_sheet(events);
        let result = work_sheet
            .get(&NaiveDate::from_ymd(2019, 1, 2))
            .unwrap()
            .get("Michel")
            .unwrap();

        // Assert
        assert_eq!(result, &actions);
    }
}