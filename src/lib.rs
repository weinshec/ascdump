use std::str::FromStr;

use thiserror::Error;

#[derive(Debug)]
struct CanFrame {
    timestamp: f32,
    bus_id: u8,
    id: u32,
    length: usize,
    payload: Vec<u8>,
}

impl CanFrame {
    fn new() -> Self {
        Self {
            timestamp: 0.0,
            bus_id: 0,
            id: 0,
            length: 0,
            payload: vec![],
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AscParseError {
    #[error("Cannot parse timestamp {str:?}")]
    InvalidTimestamp { str: String },

    #[error("Cannot parse bus id {str:?}")]
    InvalidBusId { str: String },

    #[error("Cannot parse frame id {str:?}")]
    InvalidFrameId { str: String },

    #[error("Cannot parse length field {str:?}")]
    InvalidLengthField { str: String },

    #[error("Cannot parse length field {str:?}")]
    InvalidPayload { str: String },

    #[error("Inconsistent payload length: {exp:?} != {act:?}")]
    InvalidPayloadLength { exp: usize, act: usize },

    #[error("Invalid format: '{str:?}'")]
    InvalidFormat { str: String },
}

impl FromStr for CanFrame {
    type Err = AscParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut frame = Self::new();
        let mut tokens = s.split_whitespace();

        if let Some(timestamp_token) = tokens.next() {
            frame.timestamp =
                f32::from_str(timestamp_token).map_err(|err| AscParseError::InvalidTimestamp {
                    str: err.to_string(),
                })?;
        } else {
            return Err(AscParseError::InvalidFormat { str: s.to_string() });
        }

        if let Some(bus_id_token) = tokens.next() {
            frame.bus_id =
                u8::from_str(bus_id_token).map_err(|err| AscParseError::InvalidBusId {
                    str: err.to_string(),
                })?;
        } else {
            return Err(AscParseError::InvalidFormat { str: s.to_string() });
        }

        if let Some(id_token) = tokens.next() {
            // TODO: remove trailing 'x' by using on id_token .trim_end_matches('x')
            frame.id =
                u32::from_str_radix(id_token, 16).map_err(|err| AscParseError::InvalidFrameId {
                    str: err.to_string(),
                })?;
        } else {
            return Err(AscParseError::InvalidFormat { str: s.to_string() });
        }

        if let Some(length_token) = tokens.nth(2) {
            frame.length =
                usize::from_str(length_token).map_err(|err| AscParseError::InvalidLengthField {
                    str: err.to_string(),
                })?;
            frame.payload = tokens
                .take(frame.length)
                .map(|t| u8::from_str_radix(t, 16))
                .collect::<Result<Vec<u8>, _>>()
                .map_err(|err| AscParseError::InvalidPayload {
                    str: err.to_string(),
                })?;
        } else {
            return Err(AscParseError::InvalidFormat { str: s.to_string() });
        }

        if frame.payload.len() != frame.length {
            return Err(AscParseError::InvalidPayloadLength {
                exp: frame.length,
                act: frame.payload.len(),
            });
        }

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_can_frame_from_string_timestamp() {
        let line =
            String::from("0.962604 3 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        let frame = CanFrame::from_str(&line).expect("Uncaught error while parsing");
        assert_eq!(0.962604, frame.timestamp);

        let invalid =
            String::from("0.9xxxxx 3 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        assert_eq!(true, CanFrame::from_str(&invalid).is_err());

        let invalid_length = String::from("");
        assert_eq!(true, CanFrame::from_str(&invalid_length).is_err());
    }

    #[test]
    fn parse_can_frame_from_string_bus_id() {
        let line =
            String::from("0.962604 3 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        let frame = CanFrame::from_str(&line).expect("Uncaught error while parsing");
        assert_eq!(frame.bus_id, 3);

        let invalid =
            String::from("0.962604 _ 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        assert_eq!(true, CanFrame::from_str(&invalid).is_err());

        let invalid_length = String::from("0.962604");
        assert_eq!(true, CanFrame::from_str(&invalid_length).is_err());
    }

    #[test]
    fn parse_can_frame_from_string_can_id() {
        let line =
            String::from("0.962604 3 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        let frame = CanFrame::from_str(&line).expect("Uncaught error while parsing");
        assert_eq!(frame.id, 0x368);

        let invalid =
            String::from("0.962604 3 3_8 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        assert_eq!(true, CanFrame::from_str(&invalid).is_err());

        let invalid_length = String::from("0.962604 3");
        assert_eq!(true, CanFrame::from_str(&invalid_length).is_err());
    }

    #[test]
    fn parse_can_frame_from_string_payload() {
        let line =
            String::from("0.962604 3 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        let frame = CanFrame::from_str(&line).expect("Uncaught error while parsing");
        assert_eq!(frame.length, 4);
        assert_eq!(frame.payload, vec![0xCC, 0x55, 0x01, 0x00]);

        let invalid_length_field =
            String::from("0.962604 3 368 Rx d _ cc 55 01 00 Length = 0 BitCount = 0 ID = 872");
        assert_eq!(true, CanFrame::from_str(&invalid_length_field).is_err());

        let invalid_payload =
            String::from("0.962604 3 368 Rx d 4 cc 55 __ 00 Length = 0 BitCount = 0 ID = 872");
        assert_eq!(true, CanFrame::from_str(&invalid_payload).is_err());

        let invalid_length_1 = String::from("0.962604 3 368 Rx d");
        assert_eq!(true, CanFrame::from_str(&invalid_length_1).is_err());

        let invalid_length_2 = String::from("0.962604 3 368 Rx d 4 cc");
        assert_eq!(true, CanFrame::from_str(&invalid_length_2).is_err());
    }
}
