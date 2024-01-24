use crossterm::style::Stylize;
use tokio::time::Instant;
use crate::upload_status::UploadStatus;

pub struct SharedState {
    pub(crate) files_retrieved: usize,
    pub(crate) uploaded_files: i32,
    pub(crate) corrupt_files_counter: i32,
    pub(crate) remaining_files: i32,
    pub(crate) failed_files_counter: i32,
    pub(crate) skipped_files: i32,
    pub(crate) last_processed_files: Vec<(UploadStatus, String)>,
    pub(crate) currently_uploading: Vec<(Instant, String)>,
    pub(crate) corrupt_files: Vec<(UploadStatus, String)>,
    pub(crate) failed_files: Vec<(UploadStatus, String)>,
}

impl SharedState {
    fn increment_uploaded_files(&mut self) {
        self.uploaded_files += 1;
    }
    fn increment_corrupt_files(&mut self) {
        self.corrupt_files_counter += 1;
    }
    fn increment_failed_files(&mut self) {
        self.failed_files_counter += 1;
    }

    fn increment_skipped_files(&mut self) {
        self.skipped_files += 1;
    }

    fn decrement_remaining_files(&mut self) {
        self.remaining_files -= 1;
    }

    pub(crate) fn append_to_processed_files(&mut self, content: (UploadStatus, String)) {
        self.last_processed_files.push(content.clone());
        if self.last_processed_files.len() > 20 {
            self.last_processed_files.remove(0);
        }
        match &content.0 {
            UploadStatus::Skipped => {
                self.increment_skipped_files()
            }
            UploadStatus::Failed(reason) => {
                self.append_to_failed_files(content.clone().1, *reason)
            }
            UploadStatus::Corrupt => {
                self.append_to_corrupt_files(content.clone().1)
            }
            UploadStatus::Success => {
                self.increment_uploaded_files()
            }
        }
        self.decrement_remaining_files();
    }

    pub(crate) fn set_initial_remaining_files(&mut self, number: i32) {
        self.remaining_files = number;
    }

    pub(crate) fn append_to_currently_uploading(&mut self, path: String) {
        self.currently_uploading.push((Instant::now(), path))
    }

    pub(crate) fn append_to_corrupt_files(&mut self, path: String) {
        self.corrupt_files.push((UploadStatus::Corrupt, path));
        self.increment_corrupt_files();
    }

    pub(crate) fn append_to_failed_files(&mut self, path: String, status_code: u16) {
        self.failed_files.push((UploadStatus::Failed(status_code), path));
        self.increment_failed_files();
    }

    pub(crate) fn set_files_retrieved(&mut self, amount: usize) {
        self.files_retrieved = amount;
    }

    pub(crate) fn remove_from_currently_uploading(&mut self, path: String) {
        let index = self
            .currently_uploading
            .iter()
            .position(|(_, x)| *x == path)
            .unwrap();
        self.currently_uploading.remove(index);
    }

    pub(crate) fn print_status(&self) {
        println!("Files in database: {}", self.files_retrieved);
        println!("Currently uploading:");

        for (start_time, path) in self.currently_uploading.clone().iter().rev() {
            let elapsed = start_time.elapsed();
            let hours = elapsed.as_secs() / 3600;
            let minutes = (elapsed.as_secs() % 3600) / 60;
            let seconds = elapsed.as_secs() % 60;
            println!("{:02}:{:02}:{:02}\t {}", hours, minutes, seconds, path)
        }

        println!("\nUploaded files: {}, Corrupt files: {}, Failed files: {}, Skipped files: {}, Remaining files: {}\n",
                 self.uploaded_files,
                 self.corrupt_files_counter,
                 self.failed_files_counter,
                 self.skipped_files,
                 self.remaining_files
        );

        println!("Latest processed files:");
        for (step, path) in self.last_processed_files.clone().iter().rev() {
            match step {
                UploadStatus::Failed(_) => println!("{} {}  \t {}", "FAILED -".red(), step, path),
                _ => println!("{}  \t {}", step, path)
            }
        }

        println!("\nCorrupted files:");
        for (_, path) in self.corrupt_files.clone().iter().rev() {
            println!("\t\t {}", path)
        }

        println!("\nFailed files:");
        for (status_code, path) in self.failed_files.clone().iter().rev() {
            println!("{}  \t {}", status_code, path)
        }
    }
}
