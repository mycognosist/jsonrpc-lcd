use jsonrpc_http_server::*;
use jsonrpc_http_server::jsonrpc_core::*;

use linux_embedded_hal::{Delay, Pin};
use linux_embedded_hal::sysfs_gpio::Direction;

use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};

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
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        }
    );

    lcd
}

/*
    How to safely pass the lcd object around after initialization?
     - In 'clock-mode', the display is updating each second (tick);
       how do we interrupt that tick function to write status messages?
*/

fn main() {

    let mut io = IoHandler::default();
    
    io.add_method("welcome", |_| {
        let mut lcd = lcd_init();
        lcd.write_str("Welcome to");
        lcd.set_cursor_pos(42);
        lcd.write_str("PeachCloud :)");
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
