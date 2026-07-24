//! # mount command line tool
//! A simple tool for controlling my telescope mount over serial

/// Module for talking to the arduino uno via serial
pub mod arduino {
    use clap::ValueEnum;
    use std::io::Write;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ArduinoError {
        #[error("String characters cannot be converted to ascii hex digits: {0}")]
        AsciiHexdigits(String),
        #[error("String is not correct length after conversion to ascii, {0}")]
        StringWrongLength(String),
        #[error("Failed to write to {0}")]
        WriteError(#[from] std::io::Error),
        #[error("Unsafe message: {0:?}")]
        UnsafeMessageError(Message),
    }

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
    pub enum Motor {
        /// Motor A
        A,
        /// Motor B
        B,
    }

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
    pub enum Enabled {
        /// enabled
        Enabled,
        /// disabled
        Disabled,
    }

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
    pub enum Direction {
        /// forward
        Forward,
        /// backward
        Backward,
    }

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
    pub enum Buffer {
        /// Hightime Buffer
        Hightime,
        /// Period Buffer
        Period,
    }

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub struct Message {
        motor: Motor,
        enabled: Enabled,
        direction: Direction,
        buffer: Buffer,
        value: u16,
    }

    impl Message {
        pub fn new(
            motor: Motor,
            enabled: Enabled,
            direction: Direction,
            buffer: Buffer,
            value: u16,
        ) -> Self {
            Message {
                motor,
                enabled,
                direction,
                buffer,
                value,
            }
        }

        pub fn craft_message(&self, check: bool) -> Result<[u8; 3], ArduinoError> {
            if check {
                match self.is_safe() {
                    true => (),
                    false => return Err(ArduinoError::UnsafeMessageError(*self)),
                }
            }
            let Message {
                motor,
                enabled,
                direction,
                buffer,
                value,
            } = *self;
            let mut output: [u8; 3] = [0, 0, 0];
            output[1] = (value >> 8) as u8;
            output[2] = value as u8;
            output[0] = match motor {
                Motor::A => 0,
                Motor::B => 1,
            } + match enabled {
                Enabled::Enabled => 1 << 1,
                Enabled::Disabled => 0,
            } + match buffer {
                Buffer::Hightime => 1 << 2,
                Buffer::Period => 0,
            } + match direction {
                Direction::Forward => 1 << 3,
                Direction::Backward => 0,
            };
            let mut count: u8 = output[0] ^ 0x9;
            count += ((output[1] & 0xF0) >> 4) ^ 0x4;
            count += (output[1] & 0xF) ^ 0xd;
            count += ((output[2] & 0xF0) >> 4) ^ 0x2;
            count += (output[2] & 0xF) ^ 0xa;
            output[0] += (count & 0xF) << 4;
            Ok(output)
        }

        fn is_safe(&self) -> bool {
            // speed limits
            if self.buffer == Buffer::Period
                && self.value
                    < match self.motor {
                        Motor::A => 150,
                        Motor::B => 150,
                    }
            {
                return false;
            }

            // hightime limits
            if self.buffer == Buffer::Hightime
                && self.value
                    > match self.motor {
                        Motor::A => 50,
                        Motor::B => 50,
                    }
            {
                return false;
            }

            true
        }
    }

    pub fn string_to_bytes(input: &str) -> Result<[u8; 3], ArduinoError> {
        let mut output: [u8; 3] = [0, 0, 0];
        for (i, outputi) in output.iter_mut().enumerate() {
            match input.chars().nth(i) {
                Some(x) => {
                    if x.is_ascii_hexdigit() {
                        *outputi = x.to_digit(16).unwrap() as u8; // must be safe
                    } else {
                        return Err(ArduinoError::AsciiHexdigits(input.to_string()));
                    }
                }
                None => return Err(ArduinoError::StringWrongLength(input.to_string())),
            }
        }
        Ok(output)
    }

    pub fn send_bytes(bytes: &[u8], device: &str) -> Result<(), ArduinoError> {
        match std::fs::OpenOptions::new().append(true).open(device) {
            Ok(mut file) => file.write_all(bytes)?,
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}
