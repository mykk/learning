#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;

use microbit::hal::uarte::{self, Baudrate, Parity};

mod serial_setup;
use serial_setup::UartePort;
use core::fmt::{self, Write as _};
use heapless::Vec;

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

    nb::block!(match serial.write_str("Type in something and I'll send it back to you in reverse (limited to 32 chars).\r\n") {
        Ok(_) => Ok::<(), nb::Error<fmt::Error>>(()),
        Err(er) => Err(nb::Error::Other(er))
    }).unwrap();
    nb::block!(serial.flush()).unwrap();

    loop {
        let mut buffer: Vec<u8, 32> = Vec::new();

        loop {
            let byte = nb::block!(serial.read()).unwrap();
            if byte as char == '\r' {
                break;
            }
            else {
                match buffer.push(byte) {
                    Ok(_) => {},
                    Err(_) => {
                        nb::block!( match serial.write_str("Overflow! Here's what you had so far: \r\n") {
                            Ok(_) => Ok::<(), nb::Error<fmt::Error>>(()),
                            Err(er) => Err(nb::Error::Other(er))
                        }).unwrap();

                        break;
                    }
                }
            }
        }
        
        nb::block!(buffer.iter().rev().try_for_each(|b|serial.write_byte(*b).map(|_|()))).unwrap();
        nb::block!( match serial.write_str("\r\n") {
            Ok(_) => Ok::<(), nb::Error<fmt::Error>>(()),
            Err(er) => Err(nb::Error::Other(er))
        }).unwrap();
    
        nb::block!(serial.flush()).unwrap();

        buffer.clear();
    }
}