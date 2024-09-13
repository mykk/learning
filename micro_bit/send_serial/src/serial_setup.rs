use core::fmt;
use embedded_hal::blocking::serial as bserial;
use embedded_hal::serial;
use microbit::hal::uarte::{Error as UarteError, Instance, Uarte, UarteRx, UarteTx};
use embedded_io::{Write, Read};

static mut TX_BUF: [u8; 1] = [0; 1];
static mut RX_BUF: [u8; 1] = [0; 1];

pub struct UartePort<T: Instance>{
    transmitter: UarteTx<T>,
    receiver: UarteRx<T>
}

impl<T: Instance> UartePort<T> {
    pub fn new(serial: Uarte<T>) -> UartePort<T> {
        let (tx, rx) = serial
            .split(unsafe { &mut TX_BUF }, unsafe { &mut RX_BUF })
            .unwrap();
        UartePort{transmitter: tx, receiver: rx}
    }
}

impl<T: Instance> fmt::Write for UartePort<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.transmitter.write_str(s)
    }
}

impl<T: Instance> serial::Write<u8> for UartePort<T> {
    type Error = UarteError;

    fn write(&mut self, b: u8) -> nb::Result<(), Self::Error> {
        let buf = [b];
        let transmitter = &mut self.transmitter;
        let write_result = transmitter.write(&buf);

        match write_result {
            Ok(_) => Ok(()),
            Err(er) => Err(nb::Error::Other(er))
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        match self.transmitter.flush() {
            Ok(_) => Ok(()),
            Err(er) => Err(nb::Error::Other(er))
        }
    }
}

impl<T: Instance> bserial::write::Default<u8> for UartePort<T> {}

impl<T: Instance> serial::Read<u8> for UartePort<T> {
    type Error = UarteError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut bits = [8u8;1];

        match self.receiver.read(&mut bits) {
            Ok(read) if read == 0 => Ok(bits[0]),
            Ok(_) => Err(nb::Error::Other(UarteError::RxBufferTooSmall)),
            Err(er) => Err(nb::Error::Other(er)),
        }
    }
}
