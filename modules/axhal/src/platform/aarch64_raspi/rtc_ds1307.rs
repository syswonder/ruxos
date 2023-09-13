extern crate ds1307;

use ds1307::{DateTimeAccess, Ds1307, NaiveDate};
use embedded_hal::blocking::i2c::{Write, WriteRead,SevenBitAddress};

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Capabilities {
    funcs: c_ulong,
}

pub fn i2c_write_read(
    fd: c_int,
    address: u16,
    addr_10bit: bool,
    write_buffer: &[u8],
    read_buffer: &mut [u8],
) -> Result<()> {
    // 0 length buffers may cause issues
    if write_buffer.is_empty() || read_buffer.is_empty() {
        return Ok(());
    }

    let segment_write = RdwrSegment {
        addr: address,
        flags: if addr_10bit { RDWR_FLAG_TEN } else { 0 },
        len: write_buffer.len() as u16,
        data: write_buffer.as_ptr() as usize,
    };

    let segment_read = RdwrSegment {
        addr: address,
        flags: if addr_10bit {
            RDWR_FLAG_RD | RDWR_FLAG_TEN
        } else {
            RDWR_FLAG_RD
        },
        len: read_buffer.len() as u16,
        data: read_buffer.as_mut_ptr() as usize,
    };

    let mut segments: [RdwrSegment; 2] = [segment_write, segment_read];
    let mut request = RdwrRequest {
        segments: &mut segments,
        nmsgs: 2,
    };

    parse_retval!(unsafe { ioctl(fd, REQ_RDWR, &mut request) })?;

    Ok(())
}

//use linux_embedded_hal::I2cdev;
#[derive(Debug)]
pub struct I2c {
    bus: u8,
    funcs: Capabilities,
    i2cdev: File,
    addr_10bit: bool,
    address: u16,
    // The not_sync field is a workaround to force !Sync. I2c isn't safe for
    // Sync because of ioctl() and the underlying drivers. This avoids needing
    // #![feature(optin_builtin_traits)] to manually add impl !Sync for I2c.
    not_sync: PhantomData<*const ()>,
}

impl I2c {
    /// Constructs a new `I2c`.
    ///
    /// `new` attempts to identify which I2C bus is bound to physical pins 3 (SDA)
    /// and 5 (SCL) based on the Raspberry Pi model.
    ///
    /// More information on configuring the I2C buses can be found [here].
    ///
    /// [here]: index.html#i2c-buses
    pub fn new() -> Result<I2c> {
        match DeviceInfo::new()?.model() {
            // Pi B Rev 1 uses I2C0
            Model::RaspberryPiBRev1 => I2c::with_bus(0),
            Model::RaspberryPi4B | Model::RaspberryPi400 => {
                // Pi 4B/400 could have I2C3 enabled on pins 3 and 5
                I2c::with_bus(1).or_else(|_| I2c::with_bus(3))
            }
            // Everything else should be using I2C1
            _ => I2c::with_bus(1),
        }
    }

    /// Constructs a new `I2c` using the specified bus.
    ///
    /// `bus` indicates the selected I2C bus. You'll typically want to select the
    /// bus that's bound to physical pins 3 (SDA) and 5 (SCL). On the Raspberry
    /// Pi B Rev 1, those pins are tied to bus 0. On every other Raspberry
    /// Pi model, they're connected to bus 1. Additional I2C buses are available
    /// on the Raspberry Pi 4 B and 400.
    ///
    /// More information on configuring the I2C buses can be found [here].
    ///
    /// [here]: index.html#i2c-buses
    pub fn with_bus(bus: u8) -> Result<I2c> {
        // bus is a u8, because any 8-bit bus ID could potentially
        // be configured for bit banging I2C using i2c-gpio.
        let i2cdev = OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!("/dev/i2c-{}", bus))?;

        let capabilities = ioctl::funcs(i2cdev.as_raw_fd())?;

        // Disable 10-bit addressing if it's supported
        if capabilities.addr_10bit() {
            ioctl::set_addr_10bit(i2cdev.as_raw_fd(), 0)?;
        }

        // Disable PEC if it's supported
        if capabilities.smbus_pec() {
            ioctl::set_pec(i2cdev.as_raw_fd(), 0)?;
        }

        Ok(I2c {
            bus,
            funcs: capabilities,
            i2cdev,
            addr_10bit: false,
            address: 0,
            not_sync: PhantomData,
        })
    }

    /// Returns information on the functionality supported by the underlying drivers.
    ///
    /// The returned [`Capabilities`] instance lists the available
    /// I2C and SMBus features.
    ///
    /// [`Capabilities`]: struct.Capabilities.html
    pub fn capabilities(&self) -> Capabilities {
        self.funcs
    }

    /// Returns the I2C bus ID.
    pub fn bus(&self) -> u8 {
        self.bus
    }
}

impl Write for MyI2C {
    type Error = ();

    /// Sends the outgoing data contained in `buffer` to the slave device.
    ///
    /// Sequence: START → Address + Write Bit → Outgoing Bytes → STOP
    ///
    /// Returns how many bytes were written.
    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        Ok(self.i2cdev.write(buffer)?)
    }
}

impl WriteRead for MyI2C {
    type Error = ();
    /// Sends the outgoing data contained in `write_buffer` to the slave device, and
    /// then fills `read_buffer` with incoming data.
    ///
    /// Compared to calling [`write`] and [`read`] separately, `write_read` doesn't
    /// issue a STOP condition in between the write and read operation. A repeated
    /// START is sent instead.
    ///
    /// `write_read` reads as many bytes as can fit in `read_buffer`. The maximum
    /// number of bytes in either `write_buffer` or `read_buffer` can't exceed 8192.
    ///
    /// Sequence: START → Address + Write Bit → Outgoing Bytes → Repeated START →
    /// Address + Read Bit → Incoming Bytes → STOP
    ///
    /// [`write`]: #method.write
    /// [`read`]: #method.read
    fn write_read(&self, write_buffer: &[u8], read_buffer: &mut [u8]) -> Result<()> {
        ioctl::i2c_write_read(
            self.i2cdev.as_raw_fd(),
            self.address,
            self.addr_10bit,
            write_buffer,
            read_buffer,
        )?;

        Ok(())
    }
}
pub fn test_ds1307() {
    //let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let dev = MyI2C;
    let mut rtc = Ds1307::new(dev);
    let datetime = NaiveDate::from_ymd_opt(2022, 1, 2)
        .unwrap()
        .and_hms_opt(19, 59, 58)
        .unwrap();
    rtc.set_datetime(&datetime).unwrap();
    // ...
    let datetime = rtc.datetime().unwrap();
    //println!("{datetime}");
    // This will print something like: 2022-01-02 19:59:58
}