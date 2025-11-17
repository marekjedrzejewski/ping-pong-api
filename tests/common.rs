use std::{
    env,
    io::{self, BufRead, BufReader},
    process::{Child, Command, Stdio},
};

pub fn start_server_and_wait_until_ready(db_url: &str, api_port: u16) -> Child {
    const SUCCESS_MESSAGE: &str = "Database initialized";

    start_server_and_wait_for_the_message(db_url, api_port, SUCCESS_MESSAGE)
        .expect("Failed to start server")
}

fn start_server_and_wait_for_the_message(
    db_url: &str,
    api_port: u16,
    message: &str,
) -> Result<Child, io::Error> {
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
    match wait_for_message(&mut reader, message) {
        Ok(_) => Ok(server_process),
        Err(e) => Err(e),
    }
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

fn wait_for_message<R: BufRead>(reader: &mut R, message: &str) -> Result<(), io::Error> {
    for line_result in reader.lines() {
        let line = line_result?;

        if line.contains(message) {
            return Ok(());
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Message not found"))
}
