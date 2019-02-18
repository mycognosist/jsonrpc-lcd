#[macro_use]
extern crate validator_derive;
extern crate validator;
extern crate failure;

use std::sync::{Arc, Mutex};
use serde::Deserialize;
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;
//use jsonrpc_core as rpc;
use linux_embedded_hal::sysfs_gpio::Direction;
use linux_embedded_hal::{Delay, Pin};
use validator::{Validate, ValidationError};
use failure::Fail;

#[derive(Debug, Validate, Deserialize)]
pub struct Msg {
    #[validate(range(min = "0", max = "40"))]
    position: u8,
    #[validate(length(max = "40"))]
    string: String,
}

#[derive(Debug, Fail)]
pub enum WriteError {
    #[fail(display = "validation error")]
    Invalid {}
}

impl From<WriteError> for Error {
    fn from(err: WriteError) -> Self {
        match &err {
            WriteError::Invalid {} => Error {
                code: ErrorCode::ServerError(1),
                message: err.name().unwrap().to_string(),
                data: Some(err.to_string().into())
            },
            err => Error {
                code: ErrorCode::InternalError,
                message: "internal error".into(),
                data: Some(format!("{:?}", err).into())
            }
        }
    }
}

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

fn main() {

    let lcd = Arc::new(Mutex::new(lcd_init()));
    let lcd_clone = Arc::clone(&lcd);
    let mut io = IoHandler::default();
   
    io.add_method("write", move |params: Params| {
        // todo: handle Result type explicitly - no unwraps
        let m : Msg = params.parse().unwrap();
        // todo: implement try_unlock() & handle Result explicitly
        match m.validate() {
            Ok(_) => {
                let mut lcd = lcd_clone.lock().unwrap();
                lcd.set_cursor_pos(m.position);
                lcd.write_str(&m.string);
                Ok(Value::String("success".into()))
            },
            // todo: add custom error message with Failure
            Err(e) => Err(Error::from(WriteError::Invalid{}))
        }
    });

    let lcd_clone = Arc::clone(&lcd);
    
    io.add_method("clear", move |_| {
        let mut lcd = lcd_clone.lock().unwrap();
        lcd.clear();
        Ok(Value::String("success".into()))
    });

    let lcd_clone = Arc::clone(&lcd);
    
    io.add_method("reset", move |_| {
        let mut lcd = lcd_clone.lock().unwrap();
        lcd.reset();
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
