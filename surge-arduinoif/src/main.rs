use clap::{Parser, Subcommand};
use surge_arduinoif::arduino::*;
use anyhow::Result;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// path to device to write to
    #[arg(short, long, default_value = "/dev/serial0")]
    device: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// specify and send a full message
    Full {
        #[arg(value_enum)]
        motor: Motor,
        #[arg(value_enum)]
        enabled: Enabled,
        #[arg(value_enum)]
        direction: Direction,
        #[arg(value_enum)]
        buffer: Buffer,
        #[arg()]
        value: u16,
    },
    /// specify and send raw bytes
    Raw {
        /// raw bytes in hex:
        bytes: String,
    },
    /// specify motor speed
    Speed {
        #[arg(value_enum)]
        motor: Motor,
        #[arg(value_enum)]
        direction: Direction,
        #[arg()]
        value: u16,
    },
    /// stop the motors
    Stop {
        #[arg(value_enum)]
        motor: Motor,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let device = &cli.device;
    match cli.command {
        Commands::Raw { bytes } => {
            let bytes_u8 = string_to_bytes(&bytes)?;
            send_bytes(&bytes_u8, device)?
        }
        Commands::Full {
            motor,
            enabled,
            direction,
            buffer,
            value,
        } => {
            let message = Message::new(motor, enabled, direction, buffer, value);
            let msg = message.craft_message(true)?;
            send_bytes(&msg, device)?;
        }
        Commands::Speed {
            motor,
            direction,
            value,
        } => {
            let message = Message::new(motor, Enabled::Enabled, direction, Buffer::Period, value);
            let msg = message.craft_message(true)?;
            send_bytes(&msg, device)?;
        }
        Commands::Stop { motor } => {
            let message = Message::new(
                motor,
                Enabled::Disabled,
                Direction::Forward,
                Buffer::Period,
                0xFFFF,
            );
            let msg = message.craft_message(true)?;
            send_bytes(&msg, device)?;
        }
    };
    Ok(())
}
