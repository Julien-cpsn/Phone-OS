use std::any::Any;
use std::fmt::{Debug};
use log::info;
use mousefood::prelude::{Position};
use crate::drivers::ft6206::TouchPoint;
use crate::phone::Phone;
use crate::state::PhoneState;
use crate::ui::widgets::keyboard::{KeyboardEvent};

const WIDTH: u16 = 40;
const HEIGHT: u16 = 32;

pub trait AppEvent: Any + Debug {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Debug> AppEvent for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub enum CoreEvent {
    GoBackToHomepage,
    LaunchWifiSettings
}

impl Phone<'_> {
    pub fn format_events(&mut self, events: &Vec<TouchPoint>) -> Option<Position> {
        if let Some(event) = events.first() {
            let mut x = WIDTH - (event.x * WIDTH / 240);
            let mut y = HEIGHT - 1 - (event.y * HEIGHT / 320);

            if x >= WIDTH {
                x = WIDTH - 1;
            }
            if y >= HEIGHT {
                y = HEIGHT - 1;
            }

            Some(Position::new(x, y))
        } else {
            None
        }
    }
    
    pub fn handle_touch(&mut self, touch: Position) -> anyhow::Result<()> {
        for (area, event) in &self.current_events {
            if area.contains(touch) {
                if let Some(event) = event.as_ref().as_any().downcast_ref::<CoreEvent>() {
                    info!("{:?}", event);

                    match event {
                        CoreEvent::GoBackToHomepage => self.state = PhoneState::Homepage,
                        CoreEvent::LaunchWifiSettings => self.state = PhoneState::InApp(0),
                    }
                }
                else if let Some(event) = event.as_ref().as_any().downcast_ref::<KeyboardEvent>() {
                    info!("{:?}", event);

                    self.phone_data.keyboard.as_mut().unwrap().handle_event(event);
                }
                else {
                    let state = match self.state {
                        PhoneState::InApp(index) => self.apps[index].handle_event(&mut self.phone_data, event)?,
                        _ => None
                    };

                    if let Some(state) = state {
                        self.state = state;
                    }

                }
            }
            else {
                info!("Missed {:?}", area);
            }
        }
        
        Ok(())
    }
    
    pub fn handle_first_event(&mut self) -> anyhow::Result<()> {
        let state = match self.state {
            PhoneState::InApp(index) => {
                let event = &self.current_events[0].1;
                self.apps[index].handle_event(&mut self.phone_data, event)?
            },
            _ => None
        };

        if let Some(state) = state {
            self.state = state;
        }
        
        Ok(())
    }
}