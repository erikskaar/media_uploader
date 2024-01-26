use std::fmt::{Display, Formatter};
use crossterm::style::{StyledContent, Stylize};

#[derive(Copy, Clone)]
pub enum UploadStatus {
    Skipped,
    Failed(u16),
    Corrupt,
    Success,
}

impl UploadStatus {
    pub fn get_str(self) -> StyledContent<String> {
        match self {
            UploadStatus::Skipped => String::from("SKIPPED").white(),
            UploadStatus::Failed(reason) => format!("{}", reason).red(),
            UploadStatus::Corrupt => String::from("CORRUPTED").red(),
            UploadStatus::Success => String::from("SUCCESS").green()
        }
    }
}

impl Display for UploadStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_str())
    }
}