pub mod phone;
pub mod ui;
pub mod events;
pub mod drivers;
mod state;

use display_interface_spi::SPIInterface;
use esp_idf_svc::eventloop::{EspSystemEventLoop};
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::modem::{Modem};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriverConfig};
use esp_idf_svc::hal::spi::config::{Config, MODE_0};
use esp_idf_svc::hal::units::{MegaHertz};
use esp_idf_svc::nvs::{EspDefaultNvsPartition};
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi};
use ili9341::{DisplaySize240x320, Ili9341, Orientation};
use log::info;
use mousefood::prelude::*;
use crate::drivers::ft6206::FT6206;
use crate::phone::Phone;

// Make sure large allocations go to PSRAM
#[link_section = ".psram"]
#[allow(unused)]
static mut PSRAM_BUFFER: [u8; 153600] = [0; 153600];

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello!");

    let mut phone = Phone::new();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs_default_partition = EspDefaultNvsPartition::take()?;

    /* ===== SPI ===== */

    let spi = peripherals.spi2;
    let rst = PinDriver::output(peripherals.pins.gpio4)?;
    let dc = PinDriver::output(peripherals.pins.gpio2)?;
    let sclk = peripherals.pins.gpio18;
    let sdi = peripherals.pins.gpio19; // MISO
    let sda = peripherals.pins.gpio23; // MOSI
    let cs = peripherals.pins.gpio12;

    let spi_config = Config::new()
        .baudrate(MegaHertz::from(20).into())
        .data_mode(MODE_0);

    let spi_device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sda,
        Some(sdi),
        Some(cs),
        &SpiDriverConfig::new(),
        &spi_config
    )?;

    let di = SPIInterface::new(spi_device, dc);

    /* ===== I2C ===== */

    let i2c = peripherals.i2c0;
    let scl = peripherals.pins.gpio22;
    let sda_i2c = peripherals.pins.gpio21;

    /* ===== Display & touch ===== */

    let mut display = Ili9341::new(
        di,
        rst,
        &mut Ets,
        Orientation::Portrait,
        DisplaySize240x320,
    ).unwrap();

    let mut touch_controller = FT6206::new(i2c, sda_i2c, scl)?;

    /* ===== TUI ===== */

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| phone.draw_homepage(frame))?;

    /* ===== WiFi ===== */

    let wifi = init_wifi(peripherals.modem, sysloop, nvs_default_partition)?;

    phone.wifi = Some(wifi);
    phone.event_loop(&mut terminal, &mut touch_controller)?;

    Ok(())
}

fn init_wifi(modem: Modem, sysloop: EspSystemEventLoop, nvs_default_partition: EspDefaultNvsPartition) -> anyhow::Result<EspWifi<'static>> {
    let mut wifi = EspWifi::new(
        modem,
        sysloop.clone(),
        Some(nvs_default_partition),
    )?;

    let wifi_config = Configuration::Client(ClientConfiguration::default());
    wifi.set_configuration(&wifi_config)?;
    wifi.start()?;

    Ok(wifi)
}