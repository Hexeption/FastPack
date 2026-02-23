use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

/// Create a spinner-style bar attached to the given `MultiProgress`.
pub fn spinner(mp: &MultiProgress, msg: impl Into<String>) -> ProgressBar {
    let pb = mp.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_message(msg.into());
    pb
}

/// Create a count-based progress bar attached to the given `MultiProgress`.
pub fn count_bar(mp: &MultiProgress, total: u64, msg: impl Into<String>) -> ProgressBar {
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({elapsed})")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message(msg.into());
    pb
}
