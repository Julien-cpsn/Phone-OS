use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::hal::gpio::{Gpio21, Gpio22};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_svc::hal::units::KiloHertz;
use log::{info, warn};

pub const FT6206_ADDR: u8 = 0x38;
pub const FT6206_REG_MODE: u8 = 0x00;
pub const FT6206_REG_CTRL: u8 = 0x86;
pub const FT6206_REG_GEST: u8 = 0x01;
#[allow(dead_code)]
pub const FT6206_REG_STATUS: u8 = 0x02;
pub const FT6206_REG_TOUCH1_XH: u8 = 0x03;
pub const FT6206_REG_NUM_TOUCHES: u8 = 0x02;

#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub id: u8,
    pub event: Option<TouchEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchEvent {
    Press,
    Move,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchGesture {
    Move,
    ZoomIn,
    ZoomOut,
}

pub struct FT6206<'a> {
    i2c: I2cDriver<'a>,
}

impl<'a> FT6206<'a> {
    pub fn new(i2c: I2C0, sda_i2c: Gpio21, scl: Gpio22) -> Result<Self, anyhow::Error> {
        let i2c_config = I2cConfig::new().baudrate(KiloHertz::from(100).into());
        let i2c_driver = I2cDriver::new(
            i2c,
            sda_i2c,
            scl,
            &i2c_config
        )?;

        let mut ft = Self {
            i2c: i2c_driver
        };

        // Check if device is connected by reading chip vendor ID
        let mut id_buf = [0u8; 1];
        if let Err(e) = ft.i2c.read(FT6206_ADDR, &mut id_buf, BLOCK) {
            warn!("Could not detect FT6206 touch controller: {}", e);
            return Err(anyhow::anyhow!("FT6206 not found"));
        }

        // Configure the FT6206 in normal operating mode
        ft.write_register(FT6206_REG_MODE, 0x00)?;

        // Set interrupt trigger mode to polling (0) instead of triggered (1)
        ft.write_register(FT6206_REG_CTRL, 0x00)?;

        info!("FT6206 touch controller initialized");
        Ok(ft)
    }

    fn read_register(&mut self, reg: u8, buffer: &mut [u8]) -> Result<(), anyhow::Error> {
        let data = [reg];
        self.i2c.write_read(FT6206_ADDR, &data, buffer, BLOCK)?;
        Ok(())
    }

    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), anyhow::Error> {
        let data = [reg, value];
        self.i2c.write(FT6206_ADDR, &data, BLOCK)?;
        Ok(())
    }

    pub fn read_touches(&mut self) -> Result<Vec<TouchPoint>, anyhow::Error> {
        let mut points = Vec::new();

        let mut status_buf = [0u8; 1];
        self.read_register(FT6206_REG_NUM_TOUCHES, &mut status_buf)?;
        let touch_count = status_buf[0] & 0x0F;

        if touch_count > 0 && touch_count <= 2 {
            let mut data = [0u8; 12]; // Buffer to read touch data

            // Read touch status and points
            self.read_register(FT6206_REG_TOUCH1_XH, &mut data)?;
            
            for i in 0..touch_count {
                let base_idx: usize = (i * 6) as usize;

                // Extract touch information
                let event_flag = (data[base_idx + 0] >> 6) & 0x03;
                let x = (((data[base_idx] & 0x0F) as u16) << 8) | (data[base_idx + 1] as u16);
                let y = (((data[base_idx + 2] & 0x0F) as u16) << 8) | (data[base_idx + 3] as u16);
                let id = data[base_idx + 2] >> 4;

                // Map the event type
                let event = match event_flag {
                    0 => Some(TouchEvent::Press),
                    1 => Some(TouchEvent::Release),
                    2 => Some(TouchEvent::Move),
                    _ => None,
                };

                points.push(TouchPoint { x, y, id, event });
            }
        }

        Ok(points)
    }

    pub fn get_gesture(&mut self) -> Result<Option<TouchGesture>, anyhow::Error> {
        let mut data = [0u8; 1];
        self.read_register(FT6206_REG_GEST, &mut data)?;

        let gesture = match data[0] {
            0x10 => Some(TouchGesture::Move),
            0x18 => Some(TouchGesture::ZoomIn),
            0x14 => Some(TouchGesture::ZoomOut),
            _ => None
        };

        Ok(gesture)
    }
}