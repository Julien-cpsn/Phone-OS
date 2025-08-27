use crate::events::AppEvent;
use crate::phone::Phone;
use crate::state::PhoneState;
use mousefood::prelude::{Color, Constraint, Frame, Layout, Line, Position, Rect, Span, Stylize};
use mousefood::ratatui::widgets::{Block, Borders};

impl Phone<'_> {
    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<Vec<(Rect, Box<dyn AppEvent>)>> {
        let area = frame.area();
        let state_bar_layout = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1
        };
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

        self.render_state_bar(frame, state_bar_layout);

        let content_block = Block::new()
            .borders(Borders::TOP)
            .fg(Color::DarkGray);

        frame.render_widget(content_block, content_layout);
        
        let events = match &self.state {
            PhoneState::Homepage => self.render_app_list(frame, content_inner_layout),
            PhoneState::InApp(index) => self.apps[*index].render(&mut self.phone_data, frame, content_inner_layout),
        };

        if self.phone_data.keyboard.is_some() {
            if let Ok(mut events) = events {
                let keyboard_layout = Rect {
                    x: area.x,
                    y: area.height.saturating_sub(12),
                    width: area.width,
                    height: 12
                };

                let keyboard_events = self.phone_data.keyboard.as_ref().unwrap().render(frame, keyboard_layout);
                events.extend(keyboard_events);
                Ok(events)
            }
            else {
                events
            }
        }
        else {
            events
        }
    }

    fn render_state_bar(&self, frame: &mut Frame, area: Rect) {
        let state_bar_layout = Layout::horizontal(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
            .split(area);

        let state_text = match self.state {
            PhoneState::Homepage => "PhoneOS",
            PhoneState::InApp(index) => self.apps[index].app_name(),
        };
        
        let state_span = Span::raw(state_text).into_left_aligned_line();

        enum WifiState {
            NotInitialized,
            NotConnected,
            Connecting,
            Connected(String),
        }

        let wifi_state = match &self.phone_data.wifi {
            None => WifiState::NotInitialized,
            Some(wifi) => match wifi.get_configuration() {
                Err(_) => WifiState::NotConnected,
                Ok(wifi_configuration) => match wifi_configuration.as_client_conf_ref() {
                    None => WifiState::NotConnected,
                    Some(wifi_configuration) => match wifi_configuration.ssid.is_empty() {
                        true => WifiState::NotConnected,
                        false => match wifi.is_connected() {
                            Ok(connected) => match connected {
                                true => WifiState::Connected(wifi_configuration.ssid.to_string()),
                                false => WifiState::Connecting
                            }
                            Err(_) => WifiState::NotConnected
                        }
                    }
                },
            }
        };

        let (wifi_state_text, color) = match wifi_state {
            WifiState::NotInitialized => (String::from("Not initialized"), Color::Red),
            WifiState::NotConnected => (String::from("Not connected"), Color::Red),
            WifiState::Connecting => (String::from("Connecting"), Color::Yellow),
            WifiState::Connected(text) => (text, Color::Green)
        };

        let wifi_span = Line::raw(&wifi_state_text).right_aligned().fg(color);

        frame.render_widget(state_span, state_bar_layout[0]);
        frame.render_widget(wifi_span, state_bar_layout[1]);
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