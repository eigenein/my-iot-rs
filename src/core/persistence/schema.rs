table! {
    readings (id) {
        id -> Integer,
        sensor_id -> Integer,
        timestamp -> Integer,
        value -> Binary,
    }
}

table! {
    sensors (id) {
        id -> Integer,
        sensor -> Text,
        last_reading_id -> Nullable<Integer>,
    }
}

allow_tables_to_appear_in_same_query!(
    readings,
    sensors,
);
