use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;
use std::io::Read;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct CanFrame {
    pub timestamp: f32,
    pub bus_id: u8,
    pub id: u32,
    pub length: usize,
    pub payload: Vec<u8>,
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

pub struct AscParser<R: Read> {
    lines: Lines<BufReader<R>>,
}

impl<R> AscParser<R>
where
    R: Read,
{
    pub fn new(input: R) -> Self {
        let reader = BufReader::new(input);
        Self {
            lines: reader.lines(),
        }
    }
}

impl<R> Iterator for AscParser<R>
where
    R: Read,
{
    type Item = CanFrame;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Ok(line)) = self.lines.next() {
            match CanFrame::from_str(&line) {
                Ok(frame) => return Some(frame),
                Err(_) => return self.next(),
            }
        }
        None
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
        let can_fd = s.contains("CANFD");

        if let Some(timestamp_token) = tokens.next() {
            frame.timestamp =
                f32::from_str(timestamp_token).map_err(|err| AscParseError::InvalidTimestamp {
                    str: err.to_string(),
                })?;
        } else {
            return Err(AscParseError::InvalidFormat { str: s.to_string() });
        }

        if let Some(bus_id_token) = match can_fd {
            true => tokens.nth(1),
            false => tokens.next(),
        } {
            frame.bus_id =
                u8::from_str(bus_id_token).map_err(|err| AscParseError::InvalidBusId {
                    str: err.to_string(),
                })?;
        } else {
            return Err(AscParseError::InvalidFormat { str: s.to_string() });
        }

        if let Some(id_token) = match can_fd {
            true => tokens.nth(1),
            false => tokens.next(),
        } {
            frame.id = u32::from_str_radix(id_token.trim_end_matches('x'), 16).map_err(|err| {
                AscParseError::InvalidFrameId {
                    str: err.to_string(),
                }
            })?;
        } else {
            return Err(AscParseError::InvalidFormat { str: s.to_string() });
        }

        if let Some(length_token) = match can_fd {
            true => tokens.nth(3),
            false => tokens.nth(2),
        } {
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

        let line_canfd =
            String::from("7.392600 CANFD 1 Rx 6e   1 0 6 6 ec 0a 22 ff ff f1 0 0 3000 0 0 0 0 0");
        let frame = CanFrame::from_str(&line_canfd).expect("Uncaught error while parsing");
        assert_eq!(frame.bus_id, 1);
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

        let line_canfd =
            String::from("7.392600 CANFD 1 Rx 6e   1 0 6 6 ec 0a 22 ff ff f1 0 0 3000 0 0 0 0 0");
        let frame = CanFrame::from_str(&line_canfd).expect("Uncaught error while parsing");
        assert_eq!(frame.id, 0x6e);
    }

    #[test]
    fn parse_can_frame_from_string_extended_can_id() {
        let line =
            String::from("0.962892 3 1f78c410x Rx d 8 02 00 00 00 24 00 70 03 Length = 0 BitCount = 0 ID = 528008208x");
        let frame = CanFrame::from_str(&line).expect("Uncaught error while parsing");
        assert_eq!(frame.id, 0x1f78c410);

        let line_canfd = String::from(
            "7.392600 CANFD 1 Rx 12b80210x 1 0 6 6 ec 0a 22 ff ff f1 0 0 3000 0 0 0 0 0",
        );
        let frame = CanFrame::from_str(&line_canfd).expect("Uncaught error while parsing");
        assert_eq!(frame.id, 0x12b80210);
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

        let line_canfd =
            String::from("7.392600 CANFD 1 Rx 6e   1 0 6 6 ec 0a 22 ff ff f1 0 0 3000 0 0 0 0 0");
        let frame = CanFrame::from_str(&line_canfd).expect("Uncaught error while parsing");
        assert_eq!(frame.length, 6);
        assert_eq!(frame.payload, vec![0xEC, 0x0A, 0x22, 0xFF, 0xFF, 0xF1]);
    }

    #[test]
    fn iterate_over_lines() {
        let lines = String::from(
            "0.962604 3 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872\n\
            7.392600 CANFD 1 Rx 6e   1 0 6 6 ec 0a 22 ff ff f1 0 0 3000 0 0 0 0 0",
        );

        let mut parser = AscParser::new(lines.as_bytes());

        assert_eq!(
            parser.next(),
            Some(CanFrame {
                timestamp: 0.962604,
                bus_id: 3,
                id: 0x368,
                length: 4,
                payload: vec![0xCC, 0x55, 0x01, 0x00]
            })
        );
        assert_eq!(
            parser.next(),
            Some(CanFrame {
                timestamp: 7.392600,
                bus_id: 1,
                id: 0x6e,
                length: 6,
                payload: vec![0xEC, 0x0A, 0x22, 0xFF, 0xFF, 0xF1]
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn iterate_over_lines_with_bus_filter() {
        let lines = String::from(
            "0.962604 3 368 Rx d 4 cc 55 01 00 Length = 0 BitCount = 0 ID = 872\n\
            7.392600 CANFD 1 Rx 6e   1 0 6 6 ec 0a 22 ff ff f1 0 0 3000 0 0 0 0 0",
        );

        let mut parser = AscParser::new(lines.as_bytes()).filter(|frame| frame.bus_id == 1);

        assert_eq!(
            parser.next(),
            Some(CanFrame {
                timestamp: 7.392600,
                bus_id: 1,
                id: 0x6e,
                length: 6,
                payload: vec![0xEC, 0x0A, 0x22, 0xFF, 0xFF, 0xF1]
            })
        );
        assert_eq!(parser.next(), None);
    }
}
