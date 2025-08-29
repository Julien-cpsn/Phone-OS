use crate::events::EventType;
use crate::phone::{Phone, WifiState};
use crate::state::PhoneState;
use mousefood::prelude::{Color, Frame, Line, Position, Rect, Span, Stylize};
use mousefood::ratatui::widgets::{Block, Borders};
use std::time::SystemTime;

impl Phone<'_> {
    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<EventType> {
        let area = frame.area();

        // Emulated screen debugging purpose
        frame.render_widget(Block::new().bg(Color::Rgb(15, 15, 15)), area);

        let content_layout = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 1
        };

        let content_inner_layout = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width,
            height: area.height.saturating_sub(2)
        };

        self.render_state_bar(frame);

        let content_block = Block::new()
            .borders(Borders::TOP)
            .fg(Color::DarkGray);

        frame.render_widget(content_block, content_layout);
        
        let events = match &self.state {
            PhoneState::Homepage => self.render_app_list(frame, content_inner_layout),
            PhoneState::InApp(index) => self.apps[*index].render(&mut self.phone_data, frame, content_inner_layout),
        };

        if self.phone_data.keyboard.is_some() {
            match events {
                Ok(event_type) => match event_type {
                    EventType::List(mut events) => {
                        let keyboard_events = self.phone_data.keyboard.as_ref().unwrap().render(frame);
                        events.extend(keyboard_events);
                        Ok(EventType::List(events))
                    }
                    _ => Ok(event_type)
                },
                Err(_) => events
            }
        }
        else {
            events
        }
    }

    pub fn render_state_bar(&self, frame: &mut Frame) {
        let state_rect = Rect {
            x: 0,
            y: 0,
            width: 17,
            height: 1,
        };

        let time_rect = Rect {
            x: 17,
            y: 0,
            width: 5,
            height: 1,
        };

        let wifi_rect = Rect {
            x: 22,
            y: 0,
            width: 18,
            height: 1,
        };

        let state_text = match self.state {
            PhoneState::Homepage => "PhoneOS",
            PhoneState::InApp(index) => self.apps[index].app_name(),
        };
        
        let state_line = Line::raw(state_text).left_aligned();

        let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let seconds = time.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let time_string = format!("{:02}:{:02}", hours % 24, minutes % 60);
        let time_line = Line::raw(time_string).centered().dark_gray();

        let (wifi_state_text, color) = match &self.phone_data.wifi_state {
            WifiState::NotInitialized => ("Not initialized", Color::Red),
            WifiState::NotConnected => ("Not connected", Color::Red),
            WifiState::Connecting => ("Connecting", Color::Yellow),
            WifiState::Connected(text) => (text.as_str(), Color::Green)
        };

        let wifi_line = Line::raw(wifi_state_text).right_aligned().fg(color);

        frame.render_widget(state_line, state_rect);
        frame.render_widget(time_line, time_rect);
        frame.render_widget(wifi_line, wifi_rect);
    }

    pub fn render_touch_marker(&mut self, frame: &mut Frame, touch: Position) {
        let marker = Span::raw("X").yellow();

        let touch_area = Rect::new(touch.x, touch.y, 1, 1);

        frame.render_widget(marker, touch_area);
    }
}

/*
_____  _
|  __ \| |
| |__) | |__   ___  _ __   ___
|  ___/| '_ \ / _ \| '_ \ / _ \
| |    | | | | (_) | | | |  __/
|_|___ |_|_|_|\___/|_| |_|\___|
/ __ \ / ____|
| |  | | (___
| |  | |\___ \
| |__| |____) |
\____/|_____/
*/