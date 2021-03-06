use std::{str, time::Duration};
use thiserror::Error;

/// Driver errors
#[derive(Error, Debug)]
pub enum LssDriverError {
    #[error("Failed to parse data")]
    /// Error triggered if we fail parsing incoming packet into a data structure
    PacketParsingError(String),
    #[error("Operation timed out")]
    /// Error triggered for reading timeout
    TimeoutError,
    #[error("Failed to open serial port")]
    FailedOpeningSerialPort,
    #[error("Failed to open serial port")]
    SendingError,
}

/// Colors for the LED on the servo
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LedColor {
    /// No color
    Off = 0,
    Red = 1,
    Green = 2,
    Blue = 3,
    Yellow = 4,
    Cyan = 5,
    Magenta = 6,
    White = 7,
}

impl LedColor {
    pub(crate) fn from_i32(number: i32) -> Result<LedColor, LssDriverError> {
        match number {
            0 => Ok(LedColor::Off),
            1 => Ok(LedColor::Red),
            2 => Ok(LedColor::Green),
            3 => Ok(LedColor::Blue),
            4 => Ok(LedColor::Yellow),
            5 => Ok(LedColor::Cyan),
            6 => Ok(LedColor::Magenta),
            7 => Ok(LedColor::White),
            value => Err(LssDriverError::PacketParsingError(format!(
                "Failed parsing LedColor from {}",
                value
            ))),
        }
    }
}

/// Status of the motor as responded to status query
/// If status is safe mode you can use `query_safety_status` to see more details
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MotorStatus {
    Unknown = 0,
    Limp = 1,
    FreeMoving = 2,
    Accelerating = 3,
    Traveling = 4,
    Decelerating = 5,
    Holding = 6,
    OutsideLimits = 7,
    Stuck = 8,
    Blocked = 9,
    /// You can use `query_safety_status` to see more details
    SafeMode = 10,
}

impl MotorStatus {
    pub(crate) fn from_i32(number: i32) -> Result<MotorStatus, LssDriverError> {
        match number {
            0 => Ok(MotorStatus::Unknown),
            1 => Ok(MotorStatus::Limp),
            2 => Ok(MotorStatus::FreeMoving),
            3 => Ok(MotorStatus::Accelerating),
            4 => Ok(MotorStatus::Traveling),
            5 => Ok(MotorStatus::Decelerating),
            6 => Ok(MotorStatus::Holding),
            7 => Ok(MotorStatus::OutsideLimits),
            8 => Ok(MotorStatus::Stuck),
            9 => Ok(MotorStatus::Blocked),
            10 => Ok(MotorStatus::SafeMode),
            value => Err(LssDriverError::PacketParsingError(format!(
                "Failed parsing MotorStatus from {}",
                value
            ))),
        }
    }
}

/// Reason why status mode is engaged
/// if `query_status` doesn't return `SafeMode` this should be `NoLimits`
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SafeModeStatus {
    /// Motor is not in safety mode
    NoLimits = 0,
    /// This probably means that motor was overloaded
    CurrentLimit = 1,
    /// Voltage is either too high or too low.  
    /// Query voltage to get more info
    InputVoltageOutOfRange = 2,
    /// You can query temperature to see if it's high
    TemperatureLimit = 3,
}

impl SafeModeStatus {
    pub(crate) fn from_i32(number: i32) -> Result<SafeModeStatus, LssDriverError> {
        match number {
            0 => Ok(SafeModeStatus::NoLimits),
            1 => Ok(SafeModeStatus::CurrentLimit),
            2 => Ok(SafeModeStatus::InputVoltageOutOfRange),
            3 => Ok(SafeModeStatus::TemperatureLimit),
            value => Err(LssDriverError::PacketParsingError(format!(
                "Failed parsing SafeModeStatus from {}",
                value
            ))),
        }
    }
}

/// Version of the motor
#[derive(Clone, Debug, PartialEq)]
pub enum Model {
    /// Standard model
    ST1,
    /// High speed model
    HS1,
    /// High torque model
    HT1,
    /// Other model.  
    /// Shouldn't happen unless new motors were added later
    Other(String),
}

impl Model {
    pub(crate) fn from_str(model: &str) -> Model {
        match model {
            "LSS-ST1" => Model::ST1,
            "LSS-HS1" => Model::HS1,
            "LSS-HT1" => Model::HT1,
            other => Model::Other(other.to_owned()),
        }
    }
}

/// Which status should trigger LED blinking
/// Can be combined in a list
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LedBlinking {
    NoBlinking = 0,
    Limp = 1,
    Holding = 2,
    Accelerating = 4,
    Decelerating = 8,
    Free = 16,
    Travelling = 32,
    AlwaysBlink = 63,
}

/// Modifiers used for some commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandModifier {
    /// only for P commands
    /// microseconds per second
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HSpeed28S2CSD29modifier)
    Speed(u32),
    /// Only for D or MD commands
    /// degrees per second
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HSpeed28S2CSD29modifier)
    SpeedDegrees(u32),
    /// Useful for (P, D, MD) actions
    /// in milliseconds
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HTimedmove28T29modifier)
    Timed(u32),
    /// Useful for (P, D, MD) actions
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HTimedmove28T29modifier)
    TimedDuration(Duration),
    /// Useful with (D; MD; WD; WR)
    /// in mA
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HCurrentHalt26Hold28CH29modifier)
    CurrentHold(u32),
    /// Useful with (D; MD; WD; WR.)
    /// in mA
    /// [wiki](https://www.robotshop.com/info/wiki/lynxmotion/view/lynxmotion-smart-servo/lss-communication-protocol/#HCurrentLimp28CL29modifier)
    CurrentLimp(u32),
    /// Special modifier that allows you to use modifiable commands with no modifiers
    None,
    /// Useful in case there is a new modifier that is not supported by this library
    Custom(&'static str, i32),
}

impl CommandModifier {
    pub fn to_msg(&self) -> String {
        use CommandModifier::*;
        // this is pretty inefficient because it's all using strings. But that's okay for now
        match self {
            Speed(speed) => format!("S{}", speed),
            SpeedDegrees(speed) => format!("SD{}", speed),
            Timed(time) => format!("T{}", time),
            TimedDuration(time) => format!("T{}", time.as_millis()),
            CurrentHold(current) => format!("CH{}", current),
            CurrentLimp(current) => format!("CL{}", current),
            CommandModifier::None => String::from(""),
            Custom(text, value) => format!("{}{}", text, value),
        }
    }

    pub fn vec_to_msg(modifiers: &[CommandModifier]) -> String {
        let mut buffer = String::new();
        for modifier in modifiers {
            buffer.push_str(&modifier.to_msg());
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_parses_other() {
        let model = Model::from_str("something");
        assert_eq!(model, Model::Other("something".to_owned()));
    }

    #[test]
    fn color_parse_fails() {
        let color = LedColor::from_i32(42);
        assert!(color.is_err());
    }

    #[test]
    fn colors_parse() {
        let params = vec![
            (LedColor::Off, 0),
            (LedColor::Red, 1),
            (LedColor::Green, 2),
            (LedColor::Blue, 3),
            (LedColor::Yellow, 4),
            (LedColor::Cyan, 5),
            (LedColor::Magenta, 6),
            (LedColor::White, 7),
        ];
        for (expected_color, int) in params {
            let color = LedColor::from_i32(int).unwrap();
            assert_eq!(expected_color, color);
        }
    }

    #[test]
    fn motor_status_parse_fails() {
        let status = MotorStatus::from_i32(42);
        assert!(status.is_err());
    }

    #[test]
    fn motor_status_parse() {
        let params = vec![
            (MotorStatus::Unknown, 0),
            (MotorStatus::Limp, 1),
            (MotorStatus::FreeMoving, 2),
            (MotorStatus::Accelerating, 3),
            (MotorStatus::Traveling, 4),
            (MotorStatus::Decelerating, 5),
            (MotorStatus::Holding, 6),
            (MotorStatus::OutsideLimits, 7),
            (MotorStatus::Stuck, 8),
            (MotorStatus::Blocked, 9),
            (MotorStatus::SafeMode, 10),
        ];
        for (expected_status, int) in params {
            let status = MotorStatus::from_i32(int).unwrap();
            assert_eq!(expected_status, status);
        }
    }
}
