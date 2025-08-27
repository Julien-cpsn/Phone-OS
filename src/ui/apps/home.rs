use crate::events::AppEvent;
use crate::events::CoreEvent::LaunchWifiSettings;
use crate::phone::Phone;
use crate::ui::widgets::clickable_button::BorderedButton;
use mousefood::prelude::{Constraint, Frame, Layout, Rect};
use mousefood::ratatui::widgets::Paragraph;

impl<'a> Phone<'a> {
    pub fn render_app_list(&self, frame: &mut Frame, area: Rect) -> anyhow::Result<Vec<(Rect, Box<dyn AppEvent>)>> {
        let apps_layout = Layout::vertical(vec![
            Constraint::Length(3),
        ])
            .split(area);

        for app in &self.apps {
            let bordered_button = BorderedButton(app.app_name());
            frame.render_widget(bordered_button, apps_layout[0]);
        }
        
        Ok(vec![
            (apps_layout[0], Box::new(LaunchWifiSettings))
        ])
    }

    pub fn render_homepage(&mut self, frame: &mut Frame) {
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