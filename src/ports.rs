use crate::room::RoomEvent;

pub trait RoomRepository {
    fn append_room_event(&self, id: &str, evt: RoomEvent);
    fn get_room_events(&self, id: &str) -> Vec<RoomEvent>;
}
