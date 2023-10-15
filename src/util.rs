use std::collections::HashSet;
use std::str::{from_utf8, Utf8Error};
use std::time::Duration;
use hidapi::{HidDevice, HidError};
use crate::{Kind, StreamDeckError, StreamDeckInput, StreamDeckEvent};

/// Performs get_feature_report on [HidDevice]
pub fn get_feature_report(device: &HidDevice, report_id: u8, length: usize) -> Result<Vec<u8>, HidError> {
    let mut buff = vec![0u8; length];

    // Inserting report id byte
    buff.insert(0, report_id);

    // Getting feature report
    device.get_feature_report(buff.as_mut_slice())?;

    Ok(buff)
}

/// Performs send_feature_report on [HidDevice]
pub fn send_feature_report(device: &HidDevice, payload: &[u8]) -> Result<(), HidError> {
    Ok(device.send_feature_report(payload)?)
}

/// Reads data from [HidDevice]. Blocking mode is used if timeout is specified
pub fn read_data(device: &HidDevice, length: usize, timeout: Option<Duration>) -> Result<Vec<u8>, HidError> {
    device.set_blocking_mode(timeout.is_some())?;

    let mut buf = vec![0u8; length];

    match timeout {
        Some(timeout) => device.read_timeout(buf.as_mut_slice(), timeout.as_millis() as i32),
        None => device.read(buf.as_mut_slice())
    }?;

    Ok(buf)
}

/// Writes data to [HidDevice]
pub fn write_data(device: &HidDevice, payload: &[u8]) -> Result<usize, HidError> {
    Ok(device.write(payload)?)
}

/// Extracts string from byte array, removing \0 symbols
pub fn extract_str(bytes: &[u8]) -> Result<String, Utf8Error> {
    Ok(from_utf8(bytes)?.replace('\0', "").to_string())
}

/// Flips key index horizontally, for use with Original v1 Stream Deck
pub fn flip_key_index(kind: &Kind, key: u8) -> u8 {
    let col = key % kind.column_count();
    return (key - col) + ((kind.column_count() - 1) - col);
}

/// Reads button states, empty vector if no data
pub fn read_button_states(kind: &Kind, states: &Vec<u8>) -> Vec<bool> {
    if states[0] == 0 {
        return vec![]
    }

    match kind {
        Kind::Original => {
            let mut bools = vec![];

            for i in 0..kind.key_count() {
                let flipped_i = flip_key_index(kind, i) as usize;

                bools.push(states[flipped_i + 1] != 0);
            }

            bools
        }

        Kind::Mini | Kind::MiniMk2 => {
            states[1..].iter()
                .map(|s| *s != 0)
                .collect()
        }

        _ => {
            states[4..].iter()
                .map(|s| *s != 0)
                .collect()
        }
    }
}

/// Reads lcd screen input
pub fn read_lcd_input(data: &Vec<u8>) -> Result<StreamDeckEvent, StreamDeckError> {
    let start_x = u16::from_le_bytes([data[6], data[7]]);
    let start_y = u16::from_le_bytes([data[8], data[9]]);

    match &data[4] {
        0x1 => Ok(StreamDeckEvent::TouchScreenPress(start_x, start_y)),
        0x2 => Ok(StreamDeckEvent::TouchScreenLongPress(start_x, start_y)),

        0x3 => {
            let end_x = u16::from_le_bytes([data[10], data[11]]);
            let end_y = u16::from_le_bytes([data[12], data[13]]);

            Ok(StreamDeckEvent::TouchScreenSwipe(
                (start_x, start_y),
                (end_x, end_y)
            ))
        }

        _ => Err(StreamDeckError::BadData)
    }
}

/// Reads encoder input
pub fn read_encoder_input(kind: &Kind, data: &Vec<u8>) -> Result<StreamDeckInput, StreamDeckError> {
    match &data[4] {
        0x0 => Ok(StreamDeckInput::EncoderStateChange(
            data[5..5 + kind.encoder_count() as usize].iter()
                .map(|s| *s != 0)
                .collect()
        )),

        0x1 => Ok(StreamDeckInput::EncoderTwist(
            data[5..5 + kind.encoder_count() as usize].iter()
                .map(|s| i8::from_le_bytes([*s]))
                .collect()
        )),
        _ => Err(StreamDeckError::BadData),
    }
}

/// Represents state changes for state_diff
pub enum StateChange {
    /// Rising edge at index
    Add(u8),
    /// Falling edge at index
    Remove(u8),
}

/// Generate edge triggered events from a list of states
pub fn state_diff(saved_states: &mut HashSet<u8>, new_states: &[bool]) -> Vec<StateChange> {
    new_states
        .iter()
        .enumerate()
        .filter_map(|(i, active)| {
            let i = i as u8;
            if *active != saved_states.contains(&i) {
                if *active {
                    saved_states.insert(i);
                    Some(StateChange::Add(i))
                } else {
                    saved_states.remove(&i);
                    Some(StateChange::Remove(i))
                }
            } else {
                None
            }
        })
        .collect()
}
