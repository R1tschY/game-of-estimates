CREATE TABLE room_events (
    event_id BIGSERIAL PRIMARY KEY NOT NULL,
    occurred_at timestamp NOT NULL,
    room_id UUID NOT NULL,
    event_data JSONB NOT NULL
);

CREATE INDEX room_events_idx ON room_events(room_id);