use std::thread;
use std::time::Duration;
use std::thread::sleep;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use jsonrpc_http_server::*;
use jsonrpc_http_server::jsonrpc_core::*;

use linux_embedded_hal::{Delay, Pin};
use linux_embedded_hal::sysfs_gpio::Direction;

use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};

// datetime module
use chrono::prelude::*;

// initialize the display
fn lcd_init() -> hd44780_driver::HD44780<linux_embedded_hal::Delay, hd44780_driver::bus::FourBitBus<linux_embedded_hal::Pin, linux_embedded_hal::Pin, linux_embedded_hal::Pin, linux_embedded_hal::Pin, linux_embedded_hal::Pin, linux_embedded_hal::Pin>> {

    let rs = Pin::new(484);
    let en = Pin::new(477);

    let db4 = Pin::new(483);
    let db5 = Pin::new(482);
    let db6 = Pin::new(480);
    let db7 = Pin::new(485);

    rs.export().unwrap();
    en.export().unwrap();

    db4.export().unwrap();
    db5.export().unwrap();
    db6.export().unwrap();
    db7.export().unwrap();

    rs.set_direction(Direction::Low).unwrap();
    en.set_direction(Direction::Low).unwrap();

    db4.set_direction(Direction::Low).unwrap();
    db5.set_direction(Direction::Low).unwrap();
    db6.set_direction(Direction::Low).unwrap();
    db7.set_direction(Direction::Low).unwrap();

    let mut lcd = HD44780::new_4bit(
        rs,
        en,
        db4,
        db5,
        db6,
        db7,
        Delay,
    );

    lcd.reset();

    lcd.clear();

    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Invisible,
            cursor_blink: CursorBlink::Off,
        }
    );

    lcd
}

// start displaying clock on the display
fn clock(run_clock: Arc<AtomicBool>) {
    // initialize the display
    let mut lcd = lcd_init();
    let run_clock = Arc::clone(&run_clock);
    loop {
        // check the value of the run_clock boolean expression
        // if true: update the time and write to display each second
        if run_clock.load(Ordering::SeqCst) {
            let dt = Local::now();
            // display time in hour:minute format
            let current_time = format!(
                "{h}:{m}",
                h = dt.hour(),
                m = dt.minute()
            );
            lcd.clear();
            lcd.write_str(&current_time);
            sleep(Duration::from_millis(1000));
        } else {
            break;
        };
    };
}

fn main() {

    // initialize the display
    lcd_init();

    // create a thread-safe reference-counting pointer (boolean)
    // this allows us to track the state of the clock (on / off)
    let clock_running = Arc::new(AtomicBool::new(false));
    
    // create an IoHandler for the jsonrpc server
    let mut io = IoHandler::default();

    // write welcome message to the display
    io.add_method("welcome", |_| {
        let mut lcd = lcd_init();
        lcd.write_str("Welcome to");
        lcd.set_cursor_pos(42);
        lcd.write_str("PeachCloud :)");
        Ok(Value::String("success".into()))
    });
    
    // clone the clock_running pointer to allow it to be passed into clock_on
    let run_clock = Arc::clone(&clock_running);

    // write the time to the display (clock)
    io.add_method("clock_on", move |_| {
        // set clock pointer to true (on)
        run_clock.store(true, Ordering::SeqCst);
        let run_clock = Arc::clone(&run_clock);
        thread::spawn(move || {
            // call the clock function to start displaying timer
            // pass in the clock pointer
            clock(run_clock);
        });
        Ok(Value::String("success".into()))
    });

    let run_clock = Arc::clone(&clock_running);

    io.add_method("clock_off", move |_| {
        // set clock pointer to false (off)
        // this break the loop in clock()
        run_clock.store(false, Ordering::SeqCst);
        Ok(Value::String("success".into()))
    });

    io.add_method("ap_mode", |_| {
        let mut lcd = lcd_init();
        lcd.write_str("Access-point");
        lcd.set_cursor_pos(42);
        lcd.write_str("activated");
        Ok(Value::String("success".into()))
    });

    io.add_method("client_mode", |_| {
        let mut lcd = lcd_init();
        lcd.write_str("Client-mode");
        lcd.set_cursor_pos(42);
        lcd.write_str("activated");
        Ok(Value::String("success".into()))
    });

    let server = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(
                vec![AccessControlAllowOrigin::Null]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server");


    server.wait();
}
