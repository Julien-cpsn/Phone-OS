use std::any::Any;
use std::fmt::{Debug, Formatter};
use log::info;
use mousefood::prelude::{Position};
use crate::drivers::ft6206::TouchPoint;
use crate::phone::Phone;
use crate::state::PhoneState;

const WIDTH: u16 = 40;
const HEIGHT: u16 = 32;

pub trait AppEvent: Any {
    fn as_any(&self) -> &dyn Any;
}

impl Debug for dyn AppEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_any().fmt(f)
    }
}

#[derive(Debug)]
pub enum CoreEvent {
    GoBackToHomepage,
    LaunchWifiSettings
}

impl AppEvent for CoreEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Phone<'_> {
    pub fn format_events(&mut self, events: &Vec<TouchPoint>) -> Option<Position> {
        if let Some(event) = events.first() {
            info!("{:?}", event);

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
                if let Some(event) = event.as_any().downcast_ref::<CoreEvent>() {
                    match event {
                        CoreEvent::GoBackToHomepage => self.state = PhoneState::Homepage,
                        CoreEvent::LaunchWifiSettings => self.state = PhoneState::InApp(0),
                    }
                }
                else {
                    let state = match self.state {
                        PhoneState::InApp(index) => self.apps[index].handle_event(&mut self.app_accessible, event)?,
                        _ => None
                    };

                    if let Some(state) = state {
                        self.state = state;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn handle_first_event(&mut self) -> anyhow::Result<()> {
        let state = match self.state {
            PhoneState::InApp(index) => {
                let event = &self.current_events[0].1;
                self.apps[index].handle_event(&mut self.app_accessible, event)?
            },
            _ => None
        };

        if let Some(state) = state {
            self.state = state;
        }
        
        Ok(())
    }
}