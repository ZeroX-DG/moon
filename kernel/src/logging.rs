pub fn init_logging() {
    let mut log_dir = dirs::home_dir().expect("Home directory not found");
    log_dir.push("/tmp/moon");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    log_dir.push("kernel_log.txt");
    simple_logging::log_to_file(log_dir, log::LevelFilter::Debug).expect("Can not open log file");
}
