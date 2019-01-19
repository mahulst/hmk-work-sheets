table! {
    events (id) {
        id -> Uuid,
        unique_id -> Int4,
        event_type -> Varchar,
        payload -> Text,
        timestamp -> Timestamp,
    }
}
