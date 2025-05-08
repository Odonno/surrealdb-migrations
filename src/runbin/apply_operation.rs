use color_eyre::eyre::{eyre, Result};

use crate::models::ApplyOperation;

impl ApplyOperation {
    pub fn try_from(up: Option<String>, down: Option<String>, reset: bool) -> Result<Self> {
        match (up, down, reset) {
            (Some(up), None, false) if up.is_empty() => Ok(ApplyOperation::UpSingle),
            (Some(up), None, false) => Ok(ApplyOperation::UpTo(up)),
            (None, Some(down), false) if down.is_empty() => Ok(ApplyOperation::DownSingle),
            (None, Some(down), false) => Ok(ApplyOperation::DownTo(down)),
            (None, None, true) => Ok(ApplyOperation::Reset),
            (None, None, false) => Ok(ApplyOperation::Up),
            _ => Err(eyre!("Cannot apply when multiple args in conflict")),
        }
    }
}
