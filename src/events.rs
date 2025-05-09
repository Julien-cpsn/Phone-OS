use log::info;
use crate::drivers::ft6206::TouchPoint;
use crate::phone::Phone;

impl<'a> Phone<'a> {
    pub fn handle_event(&mut self, events: &Vec<TouchPoint>) {
        for event in events {
            info!("{:?}", event);
        }
    }
}