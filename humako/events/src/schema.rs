#![allow(proc_macro_derive_resolution_fallback)]

table! {
    events (id) {
        id -> Uuid,
        unique_id -> Int4,
        event_type -> Varchar,
        payload -> Text,
        timestamp -> Timestamp,
    }
}
