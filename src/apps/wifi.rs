use std::fs::{OpenOptions};
use std::io::{Read, Write};
use std::path::{PathBuf};
use std::str::FromStr;
use crate::apps::app::{AppHandler, ClickableArea};
use crate::events::{CoreEvent, EventType};
use crate::phone::PhoneData;
use crate::state::PhoneState;
use crate::ui::widgets::clickable_button::BorderedButton;
use crate::ui::widgets::keyboard::KeyboardLayout;
use embedded_svc::wifi::AuthMethod;
use esp_idf_svc::wifi::{AccessPointInfo, ClientConfiguration, Configuration};
use log::info;
use mousefood::prelude::{Frame, Line, Rect, Stylize};
use mousefood::ratatui::widgets::{Block, Paragraph};

pub struct WifiApp {
    pub state: WifiAppState,
    pub access_points: Vec<AccessPointInfo>,
    pub known_wifis: Vec<ClientConfiguration>,
    pub known_wifis_path: PathBuf,
}

pub enum WifiAppState {
    Scanning,
    DisplayingNetworks,
    TypingPassword(usize)
}

#[derive(Debug)]
pub enum WifiEvent {
    Scan,
    DisplayNetworks,
    TypePassword(usize),
    Connect(usize)
}

impl AppHandler for WifiApp {
    type Event = WifiEvent;

    fn new() -> Self where Self: Sized {
        WifiApp {
            state: WifiAppState::Scanning,
            access_points: vec![],
            known_wifis: vec![
                // Wokwi public gateway
                ClientConfiguration {
                    ssid: heapless::String::from_str("Wokwi-GUEST").unwrap(),
                    bssid: Some([66, 19, 55, 85, 170, 1,]),
                    password: heapless::String::new(),
                    auth_method: AuthMethod::None,
                    channel: Some(
                        6,
                    ),
                    ..Default::default()
                }
            ],
            known_wifis_path: PathBuf::from("/sdcard/wifis.dat"),
        }
    }
    
    fn app_name(&self) -> &'static str {
        "WiFi settings"
    }

    fn init(&mut self, phone_data: &mut PhoneData) -> anyhow::Result<()> {
        if self.known_wifis_path.exists() {
            let mut file = OpenOptions::new()
                .read(true)
                .open(self.known_wifis_path.as_path())?;

            let mut file_content = String::new();
            file.read_to_string(&mut file_content)?;

            self.known_wifis = serde_json::from_str(&file_content)?;
        }
        else {
            info!("No known wifi found");
            self.save_known_wifis()?;
        }

        let wifi = phone_data.wifi.as_mut().unwrap();
        let aps = wifi.scan()?;
        self.access_points = aps;

        for ap in &self.access_points {
            for known_wifi in &self.known_wifis {
                if ap.ssid == known_wifi.ssid {
                    wifi.set_configuration(&Configuration::Client(known_wifi.clone()))?;
                    wifi.connect()?;
                }
            }
        }

        Ok(())
    }

    fn render(&mut self, phone_data: &mut PhoneData, frame: &mut Frame, area: Rect) -> anyhow::Result<EventType> {
        let go_back_rect = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };

        let inner_area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width - 2,
            height: area.height,
        };

        let events = match self.state {
            WifiAppState::Scanning => {
                let scanning = Line::raw("Scanning...").centered().dark_gray();
                let scanning_rect = Rect {
                    x: inner_area.x,
                    y: inner_area.y + 2,
                    width: inner_area.width,
                    height: 1,
                };
                frame.render_widget(scanning, scanning_rect);

                return Ok(EventType::Auto(Box::new(WifiEvent::Scan)));
            },
            WifiAppState::DisplayingNetworks => {
                let go_back = Line::raw("← Go back").left_aligned().dark_gray();
                frame.render_widget(go_back, go_back_rect);

                let aps = Line::raw("Access points")
                    .bold()
                    .centered();
                let aps_rect = Rect {
                    x: inner_area.x,
                    y: inner_area.y + 2,
                    width: inner_area.width,
                    height: 1,
                };
                frame.render_widget(aps, aps_rect);

                let mut events = vec![
                    ClickableArea(go_back_rect, Box::new(CoreEvent::GoBackToHomepage))
                ];

                for (index, ap) in self.access_points.iter().enumerate() {
                    let ap_span = BorderedButton(ap.ssid.as_str());
                    let rect = Rect {
                        x: inner_area.x,
                        y: inner_area.y + 4 + (3 * index as u16),
                        width: inner_area.width,
                        height: 3,
                    };
                    frame.render_widget(ap_span, rect);

                    events.push(ClickableArea(rect, Box::new(WifiEvent::TypePassword(index))));
                }

                events
            },
            WifiAppState::TypingPassword(index) => {
                let go_back = Line::raw("← Go back").left_aligned().dark_gray();
                frame.render_widget(go_back, go_back_rect);

                let ap_name = Line::raw(self.access_points[index].ssid.as_str())
                    .bold()
                    .centered();
                let ap_name_rect = Rect {
                    x: inner_area.x,
                    y: inner_area.y + 2,
                    width: inner_area.width,
                    height: 1,
                };
                frame.render_widget(ap_name, ap_name_rect);

                let password = Paragraph::new(phone_data.keyboard.as_ref().unwrap().text.clone())
                    .block(Block::bordered());
                let password_rect = Rect {
                    x: inner_area.x,
                    y: inner_area.y + 4,
                    width: inner_area.width,
                    height: 3,
                };
                frame.render_widget(password, password_rect);

                let connect = BorderedButton("Connect");
                let connect_rect = Rect {
                    x: inner_area.x,
                    y: inner_area.y + 4 + 3,
                    width: inner_area.width,
                    height: 3,
                };
                frame.render_widget(connect, connect_rect);

                vec![
                    ClickableArea(go_back_rect, Box::new(WifiEvent::DisplayNetworks)),
                    ClickableArea(connect_rect, Box::new(WifiEvent::Connect(index))),
                ]
            }
        };

        Ok(EventType::List(events))
    }

    fn handle_event(&mut self, phone_data: &mut PhoneData, event: &WifiEvent) -> anyhow::Result<Option<PhoneState>> {
        match event {
            WifiEvent::Scan => {
                let wifi = phone_data.wifi.as_mut().unwrap();
                let aps = wifi.scan()?;
                self.access_points = aps;
                self.state = WifiAppState::DisplayingNetworks;
            }
            WifiEvent::DisplayNetworks => {
                self.state = WifiAppState::DisplayingNetworks;
                phone_data.hide_keyboard();
            },
            WifiEvent::TypePassword(index) => {
                self.state = WifiAppState::TypingPassword(*index);
                phone_data.display_keyboard(KeyboardLayout::Azerty, true);
            },
            WifiEvent::Connect(index) => {
                let password_text = phone_data.keyboard.as_ref().unwrap().text.clone();
                let password: heapless::String<64> = heapless::String::try_from(password_text.as_str()).unwrap();
                phone_data.hide_keyboard();

                match &mut phone_data.wifi {
                    None => {}
                    Some(wifi) => {
                        let client_configuration = ClientConfiguration {
                            ssid: self.access_points[*index].ssid.clone(),
                            bssid: Some(self.access_points[*index].bssid.clone()),
                            auth_method: self.access_points[*index].auth_method.unwrap_or_default(),
                            password,
                            channel: Some(self.access_points[*index].channel),
                            ..Default::default()
                        };

                        self.known_wifis.push(client_configuration.clone());
                        self.save_known_wifis()?;

                        info!("Connecting to {}", client_configuration.ssid);
                        dbg!(&client_configuration);

                        wifi.set_configuration(&Configuration::Client(client_configuration))?;
                        wifi.connect()?;

                        self.state = WifiAppState::Scanning;
                        self.access_points = vec![];

                        return Ok(Some(PhoneState::Homepage));
                    }
                }
            }
        };


        Ok(None)
    }
}

impl WifiApp {
    pub fn save_known_wifis(&mut self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.known_wifis_path.as_path())?;

        let known_wifis_string = serde_json::to_string(&self.known_wifis)?;
        file.write_all(known_wifis_string.as_bytes())?;

        info!("Known wifis saved");

        Ok(())
    }
}