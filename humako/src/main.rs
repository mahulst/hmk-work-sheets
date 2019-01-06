use events::save_event;
use db_parser::TimeRowEvent;
use events::establish_connection;
use events::print_events;
use events::save_events;
use web::serve;

fn main() {
    println!("parsing...");
    let connection = establish_connection();


    let timestamp = chrono::Utc::now().naive_utc();
//    let mut events = vec![
//        TimeRowEvent {
//            id: 6,
//            employee: "Michel",
//            action: "Pauze",
//            timestamp,
//        }, TimeRowEvent {
//            id: 7,
//            employee: "Henk",
//            action: "Pauze",
//            timestamp,
//        }
//    ]
//    ;
//    let event = save_events(&connection, events);


    let events = events::get_events(&connection);
    worksheets::derive_work_sheet(events);

    serve();
}
