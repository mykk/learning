#![no_main]
#![no_std]

use core::iter;

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::Timer, pac::TIMER0,
};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    Right,
    Down,
    Left,
    Up
}
const DIRECTION_PRIORITY: [Direction; 4] = [Direction::Right, Direction::Down, Direction::Left, Direction::Up];
const MATRIX_SIZE: usize = 5;
const LEDS_OFF: [[u8; MATRIX_SIZE]; MATRIX_SIZE] =  [[0; MATRIX_SIZE]; MATRIX_SIZE];
const LEDS_ON: [[u8; MATRIX_SIZE]; MATRIX_SIZE] =  [[1; MATRIX_SIZE]; MATRIX_SIZE];

fn get_next_led(row: usize, column: usize, direction: Direction) -> Option<(usize, usize)> {
    let (next_row, next_col) = match direction {
        Direction::Right => (Some(row), column.checked_add(1).filter(|x| *x < MATRIX_SIZE)),
        Direction::Down => (row.checked_add(1).filter(|x| *x < MATRIX_SIZE), Some(column)),
        Direction::Left => (Some(row), column.checked_sub(1)),
        Direction::Up => (row.checked_sub(1), Some(column)),
    };

    next_row.zip(next_col)
}

fn get_next_led_and_dir(current_led: (usize, usize), leds: &[[u8; MATRIX_SIZE]; MATRIX_SIZE], current_dir: Direction) -> Option<((usize, usize), Direction)> {
    let (row, column) = current_led;

    iter::once(current_dir).chain(DIRECTION_PRIORITY.iter().copied()).find_map(|next_direction| {
        get_next_led(row, column, next_direction)
        .and_then(|(next_row, next_column)| {
            if leds[row][column] == leds[next_row][next_column] {
                None
            }
            else {
                Some(((next_row, next_column), next_direction))
            }
        })
    })
}

fn to_matrix_array(iter: impl Iterator<Item = (usize, usize)>) -> [(usize, usize); MATRIX_SIZE * MATRIX_SIZE] {
    let mut array = [(0usize, 0usize); MATRIX_SIZE * MATRIX_SIZE];

    iter.enumerate().filter(|(index, _)| *index < MATRIX_SIZE * MATRIX_SIZE).for_each(|(index, item)| {
        array[index] = item; 
    });

    array
}

fn build_sequence() -> [(usize, usize); MATRIX_SIZE * MATRIX_SIZE] {
    let visited_leds = [[0; MATRIX_SIZE]; MATRIX_SIZE];

    let sequence = (0..MATRIX_SIZE * MATRIX_SIZE).scan(
        (visited_leds, (0usize, 0usize), Direction::Right),
        |(visited_leds, current_led, current_dir), sequence_index| {
            visited_leds[current_led.0][current_led.1] = 1;

            if sequence_index == 0 {
                Some(*current_led)
            } else {
                get_next_led_and_dir(*current_led, &visited_leds, *current_dir).and_then(|(next_led, next_dir)| {
                    *current_led = next_led;
                    *current_dir = next_dir;
                    Some(next_led)
                })
            }
        },
    );

    to_matrix_array(sequence)
}

fn toggle_led(display: &mut Display, mut timer: &mut Timer<TIMER0>, led_display: &mut [[u8; MATRIX_SIZE]; MATRIX_SIZE], row: usize, column: usize, duration_ms: u32) {
    led_display[row][column] = if led_display[row][column] == 0 { 1 } else { 0 };
    display.show(&mut timer, *led_display, duration_ms);
}

fn spiral(board: Board, duration_ms: u32, blink_count: u32) -> ! {
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut led_display = LEDS_OFF;

    let sequence = build_sequence();
    loop {
        sequence.iter().for_each(|(row, column)| toggle_led(&mut display, &mut timer, &mut led_display, *row, *column, duration_ms));
        for _ in 0..blink_count {
            display.show(&mut timer, LEDS_OFF, duration_ms);
            display.show(&mut timer, LEDS_ON, duration_ms);    
        }
        sequence.iter().rev().for_each(|(row, column)| toggle_led(&mut display, &mut timer, &mut led_display, *row, *column, duration_ms));
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    spiral(Board::take().unwrap(), 150, 5)
}
