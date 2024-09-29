#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;

use microbit::{display, hal::{
    pac::twim0::frequency::FREQUENCY_A, twim, uarte::{self, Baudrate, Parity}, Timer}};

mod serial_setup;
use serial_setup::UartePort;
use core::fmt::{self, Write as _};
use heapless::Vec;

use lsm303agr::{AccelMode, AccelOutputDataRate, MagOutputDataRate, MagMode, Lsm303agr};

fn almost_equal(lhs: i32, rhs: i32, error: i32) -> bool {
    (lhs - rhs).abs() < error.abs()
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();
    let mut delay = Timer::new(board.TIMER0);

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_mode_and_odr(&mut delay, AccelMode::Normal, AccelOutputDataRate::Hz10).unwrap();
    sensor.set_mag_mode_and_odr(&mut delay, MagMode::HighResolution, MagOutputDataRate::Hz10).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let mut magnometer_values: Option<(i32, i32, i32)> = None;
    let mut acceleration_values: Option<(i32, i32, i32)> = None;

    loop {
        if sensor.mag_status().unwrap().xyz_new_data() {
            sensor.magnetic_field().inspect(|data| {
                let display = if let Some(previous_vals) = magnometer_values {
                    !almost_equal(previous_vals.0, data.x_nt(), 1000) || 
                    !almost_equal(previous_vals.1, data.y_nt(), 1000) ||
                    !almost_equal(previous_vals.2, data.z_nt(), 1000)
                }
                else {
                    true
                };

                if !display {
                    return;
                }

                magnometer_values = Some((data.x_nt(), data.y_nt(), data.z_nt()));

                let mut data_str = heapless::String::<128>::new();
                write!(data_str, "Magnetometer: x {} y {} z {}\r\n", data.x_nt(), data.y_nt(), data.z_nt()).unwrap();
                    
                nb::block!(match serial.write_str(data_str.as_str()) {
                    Ok(_) => Ok::<(), nb::Error<fmt::Error>>(()),
                    Err(er) => Err(nb::Error::Other(er))
                }).unwrap();
            
                nb::block!(serial.flush()).unwrap();
            }).unwrap();    
        }

        if sensor.accel_status().unwrap().xyz_new_data() {
            sensor.acceleration().inspect(|data| {
                let display = if let Some(previous_vals) = acceleration_values {
                    !almost_equal(previous_vals.0, data.x_mg(), 1000) || 
                    !almost_equal(previous_vals.1, data.y_mg(), 1000) ||
                    !almost_equal(previous_vals.2, data.z_mg(), 1000)
                }
                else {
                    true
                };

                if !display {
                    return;
                }
                acceleration_values = Some((data.x_mg(), data.y_mg(), data.z_mg()));

                let mut data_str = heapless::String::<128>::new();
                write!(data_str, "Acceleration: x {} y {} z {}\r\n", data.x_mg(), data.y_mg(), data.z_mg()).unwrap();
    
                nb::block!(match serial.write_str(data_str.as_str()) {
                    Ok(_) => Ok::<(), nb::Error<fmt::Error>>(()),
                    Err(er) => Err(nb::Error::Other(er))
                }).unwrap();
            
                nb::block!(serial.flush()).unwrap();
            }).unwrap();
        }
    }
}