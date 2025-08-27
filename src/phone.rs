use crate::apps::app::{App, AppImpl};
use crate::apps::wifi::WifiApp;
use crate::drivers::ft6206::FT6206;
use crate::events::AppEvent;
use crate::state::PhoneState;
use esp_idf_svc::wifi::EspWifi;
use mousefood::prelude::{Backend, Frame, Rect, Terminal};
use std::thread::sleep;
use std::time::Duration;
use crate::ui::widgets::keyboard::Keyboard;

pub struct Phone<'a> {
    pub state: PhoneState,
    pub should_wait_touch: bool,
    pub phone_data: PhoneData<'a>,
    pub apps: Vec<Box<dyn App>>,
    pub current_events: Vec<(Rect, Box<dyn AppEvent>)>,
}

pub struct PhoneData<'a> {
    pub wifi: Option<EspWifi<'a>>,
    pub keyboard: Option<Keyboard>
}

impl<'a> Phone<'a> {
    pub fn new() -> Self {
        Phone {
            state: PhoneState::Homepage,
            should_wait_touch: true,
            phone_data: PhoneData {
                wifi: None,
                keyboard: None,
            },
            apps: vec![
                AppImpl::<WifiApp>::new_boxed()
            ],
            current_events: vec![],
        }
    }
    
    pub fn event_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>, touch_controller: &mut FT6206) -> anyhow::Result<()> {
        terminal.draw(|frame| {
            if let Ok(current_events) = self.draw(frame) {
                self.current_events = current_events;
            }
        })?;

        loop {
            if self.should_wait_touch {
                let touches = touch_controller.read_touches()?;

                if let Some(touch) = self.format_events(&touches) {
                    self.handle_touch(touch)?;
                    terminal.draw(|frame| self.handle_draw(frame))?;
                }
            }
            else {
                self.handle_first_event()?;
                terminal.draw(|frame| self.handle_draw(frame))?;
                sleep(Duration::from_millis(250));
            }
        }
    }

    fn handle_draw(&mut self, frame: &mut Frame) {
        if let Ok(current_events) = self.draw(frame) {
            if let Some((area, _)) = current_events.first() {
                if area.x == 0 && area.y == 0 && area.width == 0 && area.height == 0 {
                    self.should_wait_touch = false;
                }
                else { 
                    self.should_wait_touch = true;
                }
            }
            else {
                self.should_wait_touch = true;
            }

            self.current_events = current_events;
        }
    }
}