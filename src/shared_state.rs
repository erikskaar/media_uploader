use tokio::time::Instant;

pub struct SharedState {
    pub(crate) uploaded_files: i32,
    pub(crate) corrupt_files: i32,
    pub(crate) remaining_files: i32,
    pub(crate) failed_files: i32,
    pub(crate) skipped_files: i32,
    pub(crate) last_started_files: Vec<String>,
    pub(crate) currently_uploading: Vec<(String, Instant)>
}

impl SharedState {
    pub(crate) fn increment_uploaded_files(&mut self) {
        self.uploaded_files += 1;
        self.decrement_remaining_files();
    }
    pub(crate) fn increment_corrupt_files(&mut self) {
        self.corrupt_files += 1;
        self.decrement_remaining_files();
    }
    pub(crate) fn increment_failed_files(&mut self) {
        self.failed_files += 1;
        self.decrement_remaining_files();
    }

    pub(crate) fn increment_skipped_files(&mut self) {
        self.skipped_files += 1;
        self.decrement_remaining_files();
    }

    fn decrement_remaining_files(&mut self) {
        self.remaining_files -= 1;
    }

    pub(crate) fn append_to_started_files(&mut self, path: String) {
        self.last_started_files.push(path);
        if self.last_started_files.len() > 20 {
            self.last_started_files.remove(0);
        }
    }

    pub(crate) fn set_initial_remaining_files(&mut self, number: i32) {
        self.remaining_files = number;
    }

    pub(crate) fn append_to_currently_uploading(&mut self, path: String) {
        self.currently_uploading.push((path, Instant::now()))
    }

    pub(crate) fn remove_from_currently_uploading(&mut self, path: String) {
        let index = self
            .currently_uploading
            .iter()
            .position(|(x,_)| *x == path)
            .unwrap();
        self.currently_uploading.remove(index);
    }

    pub(crate) fn print_status(&self) {
        println!("Currently uploading:");

        for (path, start_time) in self.currently_uploading.clone().iter().rev() {
            let elapsed = start_time.elapsed();
            let hours = elapsed.as_secs() / 3600;
            let minutes = (elapsed.as_secs() % 3600) / 60;
            let seconds = elapsed.as_secs() % 60;
            println!("{}: {:02}:{:02}:{:02}", path, hours, minutes, seconds)
        }

        println!("\nUploaded files: {}, Corrupt files: {}, Failed files: {}, Skipped files: {}, Remaining files: {}\n",
                 self.uploaded_files,
                 self.corrupt_files,
                 self.failed_files,
                 self.skipped_files,
                 self.remaining_files
        );
        println!("Latest processed files:");
        for path in self.last_started_files.clone().iter().rev() {
            println!("{}", path)
        }
    }
}
