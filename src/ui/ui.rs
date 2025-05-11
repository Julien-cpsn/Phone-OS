use crate::events::AppEvent;
use crate::phone::Phone;
use crate::state::PhoneState;
use mousefood::prelude::{Color, Constraint, Frame, Layout, Position, Rect, Span, Stylize};
use mousefood::ratatui::widgets::{Block, Borders};

impl Phone<'_> {
    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<Vec<(Rect, Box<dyn AppEvent>)>> {
        let phone_layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
            .split(frame.area());

        self.render_state_bar(frame, phone_layout[0]);

        let content_block = Block::new()
            .borders(Borders::TOP)
            .fg(Color::DarkGray);

        let inner = content_block.inner(phone_layout[1]);

        frame.render_widget(content_block, phone_layout[1]);
        
        match &self.state {
            PhoneState::Homepage => self.render_app_list(frame, inner),
            PhoneState::InApp(index) => self.apps[*index].render(&mut self.app_accessible, frame, inner),
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
            PhoneState::InApp(index) => self.apps[index].get_name()
        };
        
        let state_span = Span::raw(state_text).into_left_aligned_line();

        let wifi_state = match &self.app_accessible.wifi {
            None => None,
            Some(wifi) => match wifi.get_configuration() {
                Ok(wifi_configuration) => match wifi_configuration.as_client_conf_ref() {
                    None => None,
                    Some(wifi_configuration) => match wifi_configuration.ssid.is_empty() {
                        true => None,
                        false => Some(wifi_configuration.ssid.to_string())
                    }
                },
                Err(_) => None
            }
        };
        let wifi_span = Span::raw(wifi_state.unwrap_or(String::from("Not connected"))).into_right_aligned_line();

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