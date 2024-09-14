use core::fmt;
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

    pub fn write_byte(&mut self, b: u8) -> nb::Result<usize, UarteError>{
        self.transmitter.write(&[b]).map_err(|er| nb::Error::Other(er))
    }
    
    pub fn flush(&mut self) -> nb::Result<(), UarteError> {
        self.transmitter.flush().map_err(|er| nb::Error::Other(er))
    }

    pub fn read(&mut self) -> nb::Result<u8, UarteError> {
        let mut bits = [0u8;1];

        match self.receiver.read(&mut bits) {
            Ok(read) if read == 1 => Ok(bits[0]),
            Ok(_) => Err(nb::Error::Other(UarteError::RxBufferTooSmall)),
            Err(er) => Err(nb::Error::Other(er)),
        }
    }
}

impl<T: Instance> fmt::Write for UartePort<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.transmitter.write_str(s)
    }
}
