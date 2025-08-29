use std::any::Any;
use std::fmt::{Debug};
use async_trait::async_trait;
use log::info;
use mousefood::prelude::{Position};
use crate::apps::app::{ClickableArea};
use crate::drivers::ft6206::{TouchEvent, TouchPoint};
use crate::drivers::ili9341::{HEIGHT, WIDTH};
use crate::phone::Phone;
use crate::state::PhoneState;
use crate::ui::widgets::keyboard::{KeyboardEvent};

#[async_trait]
pub trait AppEvent: Any + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
impl<T: Any + Debug + Send + Sync> AppEvent for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub enum EventType {
    List(Vec<ClickableArea>),
    Auto(Box<dyn AppEvent>),
}

impl Default for EventType {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

#[derive(Debug)]
pub enum CoreEvent {
    GoBackToHomepage,
    LaunchApp(usize)
}

impl Phone<'_> {
    pub fn format_touches(&self, touches: &Vec<TouchPoint>) -> Option<Position> {
        if let Some(touch) = touches.first() {
            if touch.event != Some(TouchEvent::Press) {
                return None;
            } 
            
            let mut x = WIDTH - (touch.x * WIDTH / 240);
            let mut y = HEIGHT - 1 - (touch.y * HEIGHT / 320);

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

    #[allow(unused_assignments)]
    pub fn handle_touch(&mut self, touch: Position, clickable_areas: &Vec<ClickableArea>) -> anyhow::Result<Option<PhoneState>> {
        let mut touch_succeeded = false;

        for clickable_area in clickable_areas {
            let area = &clickable_area.0;
            let event = &clickable_area.1;

            if area.contains(touch) {
                touch_succeeded = true;

                let state = if let Some(event) = event.as_ref().as_any().downcast_ref::<CoreEvent>() {
                    info!("{:?}", event);

                    let state = match event {
                        CoreEvent::GoBackToHomepage => PhoneState::Homepage,
                        CoreEvent::LaunchApp(index) => PhoneState::InApp(*index),
                    };

                    Some(state)
                }
                else if let Some(event) = event.as_ref().as_any().downcast_ref::<KeyboardEvent>() {
                    info!("{:?}", event);

                    self.phone_data.keyboard.as_mut().unwrap().handle_event(event);

                    None
                }
                else {
                    let state = match self.state {
                        PhoneState::InApp(index) => self.apps[index].handle_event(&mut self.phone_data, event)?,
                        _ => None
                    };

                    state
                };

                return Ok(state);
            }
        }

        if !touch_succeeded {
            info!("Missed {:?}", touch);
        }

        Ok(None)
    }
    
    pub fn handle_auto_event(&mut self, event: &Box<dyn AppEvent>) -> anyhow::Result<Option<PhoneState>> {
        let state = match self.state {
            PhoneState::InApp(index) => self.apps[index].handle_event(&mut self.phone_data, event)?,
            _ => None
        };

        Ok(state)
    }
}