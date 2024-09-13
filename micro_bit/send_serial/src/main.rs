#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;
use embedded_hal::serial::Write;

use microbit::hal::uarte::{self, Baudrate, Parity};

mod serial_setup;
use serial_setup::UartePort;
use core::fmt::{self, Write as _};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    nb::block!(serial.write(b'x')).unwrap();    
    nb::block!(serial.flush()).unwrap();

    nb::block!( match serial.write_str("The quick brown fox jumps over the lazy dog.\r\n") {
        Ok(_) => Ok::<(), nb::Error<fmt::Error>>(()),
        Err(er) => Err(nb::Error::Other(er))
    }).unwrap();

    loop {}
}