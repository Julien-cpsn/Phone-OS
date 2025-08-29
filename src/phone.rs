use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::bounded;
use esp_idf_svc::fs::fatfs::{Fatfs};
use esp_idf_svc::hal::sd::SdCardDriver;
use esp_idf_svc::hal::sd::spi::SdSpiHostDriver;
use esp_idf_svc::hal::spi::SpiDriver;
use esp_idf_svc::io::vfs::MountedFatfs;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use crate::apps::app::{App, AppImpl};
use crate::apps::wifi::WifiApp;
use crate::events::EventType;
use crate::drivers::ft6206::FT6206;
use crate::state::PhoneState;
use crate::ui::widgets::keyboard::Keyboard;
use esp_idf_svc::wifi::EspWifi;
use log::{info};
use mousefood::prelude::{Backend, Frame, Terminal};

pub struct Phone<'a> {
    pub state: PhoneState,
    pub should_wait_touch: bool,
    pub phone_data: PhoneData,
    pub apps: Vec<Box<dyn App + 'static>>,
    pub fs: Option<MountedFatfs<Fatfs<SdCardDriver<SdSpiHostDriver<'a, SpiDriver<'a>>>>>>
}

pub struct PhoneData {
    pub wifi: Option<EspWifi<'static>>,
    pub wifi_state: WifiState,
    pub ntp: Option<EspSntp<'static>>,
    pub keyboard: Option<Keyboard>
}

#[derive(PartialEq)]
pub enum WifiState {
    NotInitialized,
    NotConnected,
    Connecting,
    Connected(String),
}

impl Phone<'_> {
    pub fn new() -> Self {
        Phone {
            state: PhoneState::Homepage,
            should_wait_touch: true,
            phone_data: PhoneData {
                wifi: None,
                wifi_state: WifiState::NotInitialized,
                ntp: None,
                keyboard: None,
            },
            apps: vec![
                AppImpl::<WifiApp>::new_boxed()
            ],
            fs: None,
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        for app in self.apps.iter_mut() {
            app.init(&mut self.phone_data)?;
        }

        Ok(())
    }

    pub fn event_loop<B: Backend>(&mut self, mut terminal: Terminal<B>, mut touch_controller: FT6206) -> anyhow::Result<()> {
        let (touch_sender, touch_receiver) = bounded(3);

        thread::spawn(move || {
            loop {
                sleep(Duration::from_millis(20));

                let touches = touch_controller.read_touches().unwrap();

                if !touches.is_empty() {
                    touch_sender.send(touches).unwrap();
                    sleep(Duration::from_millis(150));
                }
            }
        });

        let mut current_events = None;

        terminal.draw(|frame| {
            current_events = self.handle_draw(frame)
        })?;

        loop {
            self.system_check()?;

            if let Some(events) = &current_events {
                let state = match &events {
                    EventType::Auto(event) => {
                        self.handle_auto_event(event)?
                    },
                    EventType::List(clickable_areas) => {
                        if let Ok(touches) = touch_receiver.try_recv() {
                            let state = match self.format_touches(&touches) {
                                None => None,
                                Some(touch) => self.handle_touch(touch, clickable_areas)?
                            };

                            state
                        }
                        else {
                            None
                        }
                    }
                };

                if let Some(state) = state {
                    self.state = state;
                }
            }

            terminal.draw(|frame| {
                current_events = self.handle_draw(frame)
            })?;

            sleep(Duration::from_millis(100));
        }
    }
    pub fn handle_draw(&mut self, frame: &mut Frame) -> Option<EventType> {
        info!("Redraw");

        if let Ok(current_events) = self.draw(frame) {
            Some(current_events)
        }
        else {
            None
        }
    }

    pub fn system_check(&mut self) -> anyhow::Result<()> {
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

        if self.phone_data.wifi_state != wifi_state {
            match wifi_state {
                WifiState::Connected(_) if self.phone_data.ntp.is_none() => {
                    let ntp = EspSntp::new_default()?;
                    while ntp.get_sync_status() != SyncStatus::Completed {}
                    self.phone_data.ntp = Some(ntp);
                }
                _ => self.phone_data.ntp = None,
            }

            self.phone_data.wifi_state = wifi_state;
        }

        Ok(())
    }
}