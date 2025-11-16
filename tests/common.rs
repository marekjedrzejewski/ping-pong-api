use std::{
    env,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
};

pub fn start_server_and_wait_until_ready(db_url: &str, api_port: u16) -> Child {
    const SUCCESS_MESSAGE: &str = "Database initialized";

    let app_binary_path = env!("CARGO_BIN_EXE_ping_pong_api");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set by cargo");
    let profile_file_path = format!(
        "{}/target/llvm-cov-target/ping_pong_api.{}.%p.profraw",
        manifest_dir, api_port
    );

    let mut server_process = Command::new(app_binary_path)
        .env("DATABASE_URL", db_url)
        .env("RUST_LOG", "info")
        .env("SERVER_PORT", api_port.to_string())
        .env("LLVM_PROFILE_FILE", profile_file_path)
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run application");

    // env_logger logs to stderr by default, so even when looking for 'info' it will be there
    let stderr = server_process
        .stderr
        .take()
        .expect("Child process did not have a handle to stdout");
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();

    // Read the output until we see the success message
    loop {
        match reader.read_line(&mut line) {
            Ok(0) => {
                panic!("Application terminated unexpectedly before logging",);
            }
            Ok(_) => {
                if line.contains(SUCCESS_MESSAGE) {
                    break;
                }
                line.clear(); // Clear the buffer for the next line
            }
            Err(_) => panic!(),
        }
    }

    server_process
}

#[cfg(target_family = "unix")]
pub fn send_sigterm_and_wait_for_exit(mut child: Child) -> Result<(), std::io::Error> {
    let pid = child.id() as libc::pid_t;
    let signal = libc::SIGTERM;

    let result = unsafe { libc::kill(pid, signal) };

    if result == 0 {
        let _ = child.wait();
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}
