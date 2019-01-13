use events::save_event;
use db_parser::TimeRowEvent;
use events::establish_connection;
use events::print_events;
use events::save_events;
use web::serve;

fn main() {
    println!("serving...");


    serve();
}
