pub mod phone;
pub mod ui;
pub mod events;
pub mod drivers;
pub mod state;
pub mod apps;

use display_interface_spi::SPIInterface;
use esp_idf_svc::eventloop::{EspSystemEventLoop};
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{AnyInputPin, PinDriver};
use esp_idf_svc::hal::interrupt::InterruptType;
use esp_idf_svc::hal::modem::{Modem};
use esp_idf_svc::hal::prelude::{MegaHertz, Peripherals};
use esp_idf_svc::hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_svc::hal::sd::spi::SdSpiHostDriver;
use esp_idf_svc::hal::spi::{Dma, SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::io::vfs::MountedFatfs;
use esp_idf_svc::nvs::{EspDefaultNvsPartition};
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi};
use ili9341::{DisplaySize240x320, Ili9341, Orientation};
use log::info;
use mousefood::prelude::*;
use crate::drivers::ft6206::{FT6206};
use crate::phone::Phone;

// Make sure large allocations go to PSRAM
#[link_section = ".psram"]
#[allow(unused)]
static mut PSRAM_BUFFER: [u8; 4_194_304] = [0; 4_194_304];

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello!");

    let mut phone = Phone::new();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs_default_partition = EspDefaultNvsPartition::take()?;

    /* ===== HSPI ===== */

    let spi2 = peripherals.spi2;
    let rst = PinDriver::output(peripherals.pins.gpio4)?;
    let dc = PinDriver::output(peripherals.pins.gpio2)?;
    let sclk = peripherals.pins.gpio14;
    //let sdi = peripherals.pins.gpio19; // MISO
    let sda = peripherals.pins.gpio13; // MOSI
    let cs = peripherals.pins.gpio15;

    let hspi_config = SpiConfig::new()
        .baudrate(MegaHertz::from(40).into())
        .write_only(true)
        .polling(false);

    let hspi_driver_config = SpiDriverConfig::new()
        .intr_flags(InterruptType::Level3.into());

    let hspi_device = SpiDeviceDriver::new_single(
        spi2,
        sclk,
        sda,
        None::<AnyInputPin>,
        Some(cs),
        &hspi_driver_config,
        &hspi_config
    )?;

    let di = SPIInterface::new(hspi_device, dc);

    /* ===== VSPI ===== */

    let spi3 = peripherals.spi3;
    let sclk = peripherals.pins.gpio18;
    let sdo = peripherals.pins.gpio19; // MISO
    let sdi = peripherals.pins.gpio23; // MOSI
    let cs = peripherals.pins.gpio5;

    let vspi_driver_config = SpiDriverConfig::new()
        .dma(Dma::Channel2(4096));

    let vspi_driver = SpiDriver::new(
        spi3,
        sclk,
        sdo,
        Some(sdi),
        &vspi_driver_config
    )?;

    let mut sdcard_configuration = SdCardConfiguration::new();
    sdcard_configuration.speed_khz = 4_000;

    let sd_card_driver = SdCardDriver::new_spi(
        SdSpiHostDriver::new(
            vspi_driver,
            Some(cs), //None::<AnyOutputPin>,
            None::<AnyInputPin>,
            None::<AnyInputPin>,
            None::<AnyInputPin>,
            None,
        )?,
        &sdcard_configuration,
    )?;

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

    let touch_controller = FT6206::new(i2c, sda_i2c, scl)?;

    /* ===== TUI ===== */

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| phone.render_homepage(frame))?;

    /* ===== WiFi ===== */

    let wifi = init_wifi(peripherals.modem, sysloop, nvs_default_partition)?;

    phone.phone_data.wifi = Some(wifi);

    /* ===== SD Card ===== */

    phone.fs = Some(MountedFatfs::mount(Fatfs::new_sdcard(0, sd_card_driver)?, "/sdcard", 10)?);

    /* ===== Main loop ===== */

    phone.init()?;
    phone.event_loop(terminal, touch_controller)?;

    Ok(())
}

fn init_wifi(modem: Modem, sysloop: EspSystemEventLoop, nvs_default_partition: EspDefaultNvsPartition) -> anyhow::Result<EspWifi<'static>> {
    let mut wifi = EspWifi::new(
        modem,
        sysloop.clone(),
        Some(nvs_default_partition),
    )?;

    let client_config = ClientConfiguration::default();
    let wifi_config = Configuration::Client(client_config);
    wifi.set_configuration(&wifi_config)?;
    wifi.start()?;

    Ok(wifi)
}