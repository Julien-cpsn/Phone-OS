use crate::events::{CoreEvent, EventType};
use crate::phone::Phone;
use crate::ui::widgets::clickable_button::BorderedButton;
use mousefood::prelude::{Color, Frame, Rect, Stylize};
use mousefood::ratatui::widgets::{Block, Paragraph};
use crate::apps::app::{ClickableArea};

impl Phone {
    pub fn render_app_list(&self, frame: &mut Frame, area: Rect) -> anyhow::Result<EventType> {
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width - 2,
            height: area.height,
        };

        let mut events = Vec::new();

        for (index, app) in self.apps.iter().enumerate() {
            let app_rect = Rect {
                x: inner_area.x,
                y: inner_area.y + (index as u16 * 3),
                width: inner_area.width,
                height: 3,
            };

            let bordered_button = BorderedButton(app.app_name());
            frame.render_widget(bordered_button, app_rect);

            events.push(ClickableArea(app_rect, Box::new(CoreEvent::LaunchApp(index))));
        }
        
        Ok(EventType::List(events))
    }

    pub fn render_homepage(&mut self, frame: &mut Frame) {
        // Emulated screen debugging purpose
        frame.render_widget(Block::new().bg(Color::Rgb(15, 15, 15)), frame.area());

        let logo = Paragraph::new(vec![
            " _____  _".into(),
            "|  __ \\| |".into(),
            "| |__) | |__   ___  _ __   ___".into(),
            "|  ___/| '_ \\ / _ \\| '_ \\ / _ \\".into(),
            "| |    | | | | (_) | | | |  __/".into(),
            "|_|__  |_|_|_|\\___/|_| |_|\\___|".into(),
            "/  _ \\  / ___|".into(),
            "| | | || (__".into(),
            "| | | | \\__ \\".into(),
            "| |_| | ___) |".into(),
            "\\____/ |____/".into(),
            "Loading...".into(),
        ]);
        frame.render_widget(logo, frame.area());
    }
}