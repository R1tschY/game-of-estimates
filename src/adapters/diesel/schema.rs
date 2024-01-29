// @generated automatically by Diesel CLI.

diesel::table! {
    rooms_events (event_id) {
        event_id -> Int8,
        #[max_length = 32]
        room_id -> Varchar,
        event_data -> Jsonb,
    }
}
