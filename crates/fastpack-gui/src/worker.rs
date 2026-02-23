/// Messages sent from the background packer thread to the UI thread.
pub enum WorkerMessage {
    Started,
    Progress { done: usize, total: usize },
    Finished,
    Failed(String),
}

/// Commands sent from the UI thread to the background packer thread.
pub enum WorkerCommand {
    Pack,
    Cancel,
}
