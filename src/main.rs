#[macro_use]
extern crate validator_derive;
extern crate failure;
extern crate validator;

use failure::Fail;
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};
use jsonrpc_http_server::jsonrpc_core::types::error::Error;
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;
use linux_embedded_hal::sysfs_gpio::Direction;
use linux_embedded_hal::{Delay, Pin};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use validator::{Validate, ValidationErrors};

// define the Msg struct for receiving display write commands
#[derive(Debug, Validate, Deserialize)]
pub struct Msg {
    #[validate(range(min = "0", max = "40", message = "position not in range 0-40"))]
    position: u8,
    #[validate(length(max = "40", message = "string length > 40 characters"))]
    string: String,
}

#[derive(Debug, Fail)]
pub enum WriteError {
    #[fail(display = "validation error")]
    Invalid { e: ValidationErrors },

    #[fail(display = "missing expected parameters")]
    MissingParams { e: Error },
}

impl From<WriteError> for Error {
    fn from(err: WriteError) -> Self {
        match &err {
            WriteError::Invalid { e } => {
                let err_clone = e.clone();
                // extract error from ValidationErrors
                let field_errs = err_clone.field_errors();
                let checks = vec!["position", "string"];
                // check source of validation err: "position" or "string"
                for &error in &checks {
                    let validation_err = field_errs.get(&error);
                    if validation_err.is_some() {
                        let validation_err = validation_err.unwrap();
                        let err_msg = &validation_err[0].message;
                        let em = err_msg.clone();
                        let em = em.expect("failed to unwrap error msg");
                        return Error {
                            code: ErrorCode::ServerError(1),
                            message: "validation error".into(),
                            data: Some(format!("{}", em).into()),
                        };
                    }
                }
                Error {
                    code: ErrorCode::ServerError(1),
                    message: "validation error".into(),
                    data: Some(format!("{:?}", e).into()),
                }
            }
            WriteError::MissingParams { e } => Error {
                code: ErrorCode::ServerError(-32602),
                message: "invalid params".into(),
                data: Some(format!("{}", e.message).into()),
            },
            err => Error {
                code: ErrorCode::InternalError,
                message: "internal error".into(),
                data: Some(format!("{:?}", err).into()),
            },
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
        // parse parameters and match on result
        let m: Result<Msg> = params.parse();
        match m {
            // if result contains parameters, unwrap and validate
            Ok(_) => {
                let m: Msg = m.unwrap();
                match m.validate() {
                    Ok(_) => {
                        let mut lcd = lcd_clone.lock().unwrap();
                        lcd.set_cursor_pos(m.position);
                        lcd.write_str(&m.string);
                        Ok(Value::String("success".into()))
                    }
                    Err(e) => Err(Error::from(WriteError::Invalid { e })),
                }
            }
            // if result contains an error, throw missing params error
            Err(e) => Err(Error::from(WriteError::MissingParams { e })),
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
