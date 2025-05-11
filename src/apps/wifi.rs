use crate::apps::app::AppHandler;
use crate::events::{AppEvent, CoreEvent};
use crate::phone::AppAccessible;
use crate::ui::widgets::clickable_button::BorderedButton;
use esp_idf_svc::wifi::{AccessPointInfo, ClientConfiguration, Configuration};
use mousefood::prelude::{Constraint, Frame, Layout, Line, Rect, Stylize};
use std::any::Any;
use crate::state::PhoneState;

pub struct WifiApp {
    pub should_scan: bool,
    pub access_points: Vec<AccessPointInfo>
}

pub enum WifiEvent {
    Scan,
    Connect(usize)
}

impl AppEvent for WifiEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AppHandler for WifiApp {
    type Event = WifiEvent;

    fn new() -> Self where Self: Sized {
        WifiApp {
            should_scan: true,
            access_points: vec![],
        }
    }
    
    fn get_name(&self) -> &'static str {
        "WiFi settings"
    }

    fn render(&mut self, _: &mut AppAccessible, frame: &mut Frame, area: Rect) -> anyhow::Result<Vec<(Rect, Box<dyn AppEvent>)>> {
        let app_layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
            .split(area);

        if self.should_scan {
            let loading = Line::raw("Loading...").centered().dark_gray();
            frame.render_widget(loading, app_layout[2]);

            return Ok(vec![
                (Rect::default(), Box::new(WifiEvent::Scan))
            ]);
        }

        let go_back = Line::raw("← Go back").left_aligned().dark_gray();
        frame.render_widget(go_back, app_layout[0]);

        let mut aps_contraints = vec![];
        for _ in &self.access_points {
            aps_contraints.push(Constraint::Length(3));
        }
        let aps_layout = Layout::vertical(aps_contraints).split(app_layout[2]);

        let mut events: Vec<(Rect, Box<dyn AppEvent>)> = vec![
            (app_layout[0], Box::new(CoreEvent::GoBackToHomepage))
        ];

        for (index, ap) in self.access_points.iter().enumerate() {
            let ap_span = BorderedButton(ap.ssid.as_str());
            frame.render_widget(ap_span, aps_layout[index]);

            events.push((aps_layout[index], Box::new(WifiEvent::Connect(index))));
        }

        Ok(events)
    }

    fn handle_event(&mut self, app_accessible: &mut AppAccessible, event: &WifiEvent) -> anyhow::Result<Option<PhoneState>> {
        match event {
            WifiEvent::Scan => {
                let wifi = app_accessible.wifi.as_mut().unwrap();
                let aps = wifi.scan()?;
                self.access_points = aps;
                self.should_scan = false;
            }
            WifiEvent::Connect(index) => match &mut app_accessible.wifi {
                None => {}
                Some(wifi) => {
                    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
                        ssid: self.access_points[*index].ssid.clone(),
                        bssid: Some(self.access_points[*index].bssid.clone()),
                        auth_method: self.access_points[*index].auth_method.unwrap_or_default(),
                        password: Default::default(),
                        channel: Some(self.access_points[*index].channel),
                        ..Default::default()
                    }))?;

                    wifi.connect()?;

                    return Ok(Some(PhoneState::Homepage));
                }
            },
        };


        Ok(None)
    }
}