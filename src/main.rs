use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use chrono::prelude::*;
use crossbeam_channel::tick;
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;
use linux_embedded_hal::sysfs_gpio::Direction;
use linux_embedded_hal::{Delay, Pin};

// led heartbeat
fn heartbeat_led() {
    let hb = Pin::new(462);

    hb.export().unwrap();
    hb.set_direction(Direction::Low).unwrap();

    loop {
        hb.set_value(0).unwrap();
        sleep(Duration::from_millis(500));
        hb.set_value(1).unwrap();
        sleep(Duration::from_millis(500));
    }
}

// initialize the display
fn lcd_init() -> hd44780_driver::HD44780<
    linux_embedded_hal::Delay,
    hd44780_driver::bus::FourBitBus<
        linux_embedded_hal::Pin,
        linux_embedded_hal::Pin,
        linux_embedded_hal::Pin,
        linux_embedded_hal::Pin,
        linux_embedded_hal::Pin,
        linux_embedded_hal::Pin,
    >,
> {
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

    let mut lcd = HD44780::new_4bit(rs, en, db4, db5, db6, db7, Delay);

    lcd.reset();

    lcd.clear();

    lcd.set_display_mode(DisplayMode {
        display: Display::On,
        cursor_visibility: Cursor::Invisible,
        cursor_blink: CursorBlink::Off,
    });

    lcd
}

// loop to write clock to display, updates every second
fn clock(
    run_clock: Arc<AtomicBool>,
    lcd_clone: Arc<
        Mutex<
            hd44780_driver::HD44780<
                linux_embedded_hal::Delay,
                hd44780_driver::bus::FourBitBus<
                    linux_embedded_hal::Pin,
                    linux_embedded_hal::Pin,
                    linux_embedded_hal::Pin,
                    linux_embedded_hal::Pin,
                    linux_embedded_hal::Pin,
                    linux_embedded_hal::Pin,
                >,
            >,
        >,
    >,
) {
    let timer = tick(Duration::from_millis(1000));
    let run_clock = Arc::clone(&run_clock);
    let mut lcd = lcd_clone.lock().unwrap();

    loop {
        // get time & print to display if run_clock is true
        if run_clock.load(Ordering::SeqCst) {
            timer.recv().unwrap();
            let dt = Local::now();
            let current_time = format!(
                "{:02}:{:02}:{:02}",
                dt.hour(),
                dt.minute(),
                dt.second()
            );
            lcd.clear();
            lcd.write_str(&current_time);
        } else {
            // break out of loop once run_clock is set to false
            // this released the lock on the lcd mutex
            break;
        };
    }
}

fn main() {

    thread::spawn(|| {
        heartbeat_led();
    });

    let lcd = Arc::new(Mutex::new(lcd_init()));
    let clock_running = Arc::new(AtomicBool::new(false));
    let mut io = IoHandler::default();
    let lcd_clone = Arc::clone(&lcd);

    io.add_method("welcome", move |_| {
        let mut lcd = lcd_clone.try_lock();
        if let Ok(ref mut lcd) = lcd {
            lcd.clear();
            lcd.write_str("Welcome to");
            lcd.set_cursor_pos(42);
            lcd.write_str("PeachCloud :)");
            Ok(Value::String("success".into()))
        } else {
            Err(Error::new(ErrorCode::ServerError(-34)))
        }
    });

    let lcd_clone = Arc::clone(&lcd);

    io.add_method("ap_mode", move |_| {
        let mut lcd = lcd_clone.try_lock();
        if let Ok(ref mut lcd) = lcd {
            lcd.clear();
            lcd.write_str("Access-point");
            lcd.set_cursor_pos(42);
            lcd.write_str("activated");
            Ok(Value::String("success".into()))
        } else {
            Err(Error::new(ErrorCode::ServerError(-34)))
        }
    });

    let lcd_clone = Arc::clone(&lcd);

    io.add_method("client_mode", move |_| {
        let mut lcd = lcd_clone.try_lock();
        if let Ok(ref mut lcd) = lcd {
            lcd.clear();
            lcd.write_str("Client-mode");
            lcd.set_cursor_pos(42);
            lcd.write_str("activated");
            Ok(Value::String("success".into()))
        } else {
            Err(Error::new(ErrorCode::ServerError(-34)))
        }
    });

    let lcd_clone = Arc::clone(&lcd);
    let run_clock = Arc::clone(&clock_running);

    io.add_method("clock_on", move |_| {
        run_clock.store(true, Ordering::SeqCst);
        let run_clock = Arc::clone(&run_clock);
        let lcd_clone = Arc::clone(&lcd_clone);
        thread::spawn(move || {
            clock(run_clock, lcd_clone);
        });
        Ok(Value::String("success".into()))
    });

    let run_clock = Arc::clone(&clock_running);

    io.add_method("clock_off", move |_| {
        run_clock.store(false, Ordering::SeqCst);
        Ok(Value::String("success".into()))
    });

    let server = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
        ]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server");

    server.wait();
}
