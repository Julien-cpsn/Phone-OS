use crate::drivers::ft6206::FT6206;
use esp_idf_svc::wifi::EspWifi;
use mousefood::prelude::{Backend, Terminal};
use crate::state::PhoneState;

pub struct Phone<'a> {
    pub state: PhoneState,
    pub wifi: Option<EspWifi<'a>>
}

impl<'a> Phone<'a> {
    pub fn new() -> Self {
        Phone {
            state: PhoneState::Homepage,
            wifi: None,
        }
    }
    
    pub fn event_loop<T: Backend>(&mut self, terminal: &mut Terminal<T>, touch_controller: &mut FT6206) -> anyhow::Result<()> {
        terminal.draw(|frame| self.draw(frame, vec![]))?;

        loop {
            let touches = touch_controller.read_touches()?;

            if !touches.is_empty() {
                self.handle_event(&touches);
                terminal.draw(|frame| self.draw(frame, touches))?;
            }
        }
    }
}