use crate::{serve::Serve, BootArgs};
use anyhow::Result;
use std::{
    fs::File,
    path::{Path, PathBuf},
    process,
};

#[cfg(unix)]
use daemonize::Daemonize;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Cross-platform paths
#[cfg(unix)]
const PID_PATH: &str = "/var/run/auth.pid";
#[cfg(unix)]
const DEFAULT_STDOUT_PATH: &str = "/var/run/auth.out";
#[cfg(unix)]
const DEFAULT_STDERR_PATH: &str = "/var/run/auth.err";

#[cfg(windows)]
fn get_temp_dir() -> PathBuf {
    std::env::temp_dir()
}

#[cfg(windows)]
fn get_pid_path() -> PathBuf {
    get_temp_dir().join("devicecheck.pid")
}

#[cfg(windows)]
fn get_stdout_path() -> PathBuf {
    get_temp_dir().join("devicecheck.out")
}

#[cfg(windows)]
fn get_stderr_path() -> PathBuf {
    get_temp_dir().join("devicecheck.err")
}

/// Get the pid of the daemon
fn get_pid() -> Option<String> {
    let pid_path = get_pid_path_cross_platform();
    if let Ok(data) = std::fs::read(&pid_path) {
        let binding = String::from_utf8(data).expect("pid file is not utf8");
        return Some(binding.trim().to_string());
    }
    None
}

fn get_pid_path_cross_platform() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from(PID_PATH)
    }
    #[cfg(windows)]
    {
        get_pid_path()
    }
}

fn get_stdout_path_cross_platform() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from(DEFAULT_STDOUT_PATH)
    }
    #[cfg(windows)]
    {
        get_stdout_path()
    }
}

fn get_stderr_path_cross_platform() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from(DEFAULT_STDERR_PATH)
    }
    #[cfg(windows)]
    {
        get_stderr_path()
    }
}

/// Check if the current user has appropriate permissions
#[cfg(unix)]
fn check_root() {
    if !nix::unistd::Uid::effective().is_root() {
        println!("You must run this executable with root permissions");
        std::process::exit(-1)
    }
}

#[cfg(windows)]
fn check_root() {
    // On Windows, we don't require administrator privileges for daemon operations
    // The service will run with current user privileges
}

/// Start the daemon
pub fn start(args: BootArgs) -> Result<()> {
    if let Some(pid) = get_pid() {
        println!("devicecheck is already running with pid: {}", pid);
        return Ok(());
    }

    #[cfg(unix)]
    {
        start_unix_daemon(args)
    }
    #[cfg(windows)]
    {
        start_windows_service(args)
    }
}

#[cfg(unix)]
fn start_unix_daemon(args: BootArgs) -> Result<()> {
    check_root();

    let pid_path = get_pid_path_cross_platform();
    let stdout_path = get_stdout_path_cross_platform();
    let stderr_path = get_stderr_path_cross_platform();

    let pid_file = File::create(&pid_path)?;
    pid_file.set_permissions(std::fs::Permissions::from_mode(0o755))?;

    let stdout = File::create(&stdout_path)?;
    stdout.set_permissions(std::fs::Permissions::from_mode(0o755))?;

    let stderr = File::create(&stderr_path)?;
    stderr.set_permissions(std::fs::Permissions::from_mode(0o755))?;

    let mut daemonize = Daemonize::new()
        .pid_file(&pid_path)
        .chown_pid_file(true)
        .umask(0o777)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    if let Ok(user) = std::env::var("SUDO_USER") {
        if let Ok(Some(real_user)) = nix::unistd::User::from_name(&user) {
            daemonize = daemonize
                .user(real_user.name.as_str())
                .group(real_user.gid.as_raw());
        }
    }

    if let Some(err) = daemonize.start().err() {
        eprintln!("Error: {err}");
        std::process::exit(-1)
    }

    Serve(args).run()
}

#[cfg(windows)]
fn start_windows_service(args: BootArgs) -> Result<()> {
    check_root();

    let pid_path = get_pid_path_cross_platform();
    let stdout_path = get_stdout_path_cross_platform();
    let stderr_path = get_stderr_path_cross_platform();

    // Create pid file with current process id
    let current_pid = process::id();
    std::fs::write(&pid_path, current_pid.to_string())?;

    println!("Starting devicecheck service on Windows...");
    println!("PID file: {}", pid_path.display());
    println!("Stdout log: {}", stdout_path.display());
    println!("Stderr log: {}", stderr_path.display());

    // Redirect stdout and stderr to files
    let _stdout = File::create(&stdout_path)?;
    let _stderr = File::create(&stderr_path)?;

    // Start the server
    match Serve(args).run() {
        Ok(_) => {
            // Clean up pid file on successful exit
            let _ = std::fs::remove_file(&pid_path);
            Ok(())
        }
        Err(e) => {
            // Clean up pid file on error
            let _ = std::fs::remove_file(&pid_path);
            Err(e)
        }
    }
}

/// Stop the daemon
pub fn stop() -> Result<()> {
    #[cfg(unix)]
    {
        stop_unix_daemon()
    }
    #[cfg(windows)]
    {
        stop_windows_service()
    }
}

#[cfg(unix)]
fn stop_unix_daemon() -> Result<()> {
    use nix::sys::signal;
    use nix::unistd::Pid;

    check_root();

    if let Some(pid) = get_pid() {
        let pid = pid.parse::<i32>()?;
        for _ in 0..360 {
            if signal::kill(Pid::from_raw(pid), signal::SIGINT).is_err() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1))
        }
        let pid_path = get_pid_path_cross_platform();
        let _ = std::fs::remove_file(&pid_path);
    }

    Ok(())
}

#[cfg(windows)]
fn stop_windows_service() -> Result<()> {
    check_root();

    if let Some(pid_str) = get_pid() {
        let pid: u32 = pid_str.parse()?;
        println!("Attempting to stop devicecheck service with PID: {}", pid);

        // On Windows, we can try to terminate the process
        // This is a simplified approach - in production you might want to use Windows services API
        let output = process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("Successfully stopped devicecheck service");
                    let pid_path = get_pid_path_cross_platform();
                    let _ = std::fs::remove_file(&pid_path);
                } else {
                    println!("Failed to stop service: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                println!("Error executing taskkill: {}", e);
                // Try to remove pid file anyway
                let pid_path = get_pid_path_cross_platform();
                let _ = std::fs::remove_file(&pid_path);
            }
        }
    } else {
        println!("No running devicecheck service found");
    }

    Ok(())
}

/// Restart the daemon
pub fn restart(args: BootArgs) -> Result<()> {
    stop()?;
    start(args)
}

/// Show the status of the daemon
pub fn status() -> Result<()> {
    match get_pid() {
        Some(pid) => {
            println!("devicecheck is running with pid: {}", pid);
            Ok(())
        }
        None => anyhow::bail!("devicecheck is not running"),
    }
}

/// Show the log of the daemon
pub fn log() -> Result<()> {
    fn read_and_print_file(file_path: &Path, placeholder: &str) -> Result<()> {
        if !file_path.exists() {
            return Ok(());
        }

        // Check if the file is empty before opening it
        let metadata = std::fs::metadata(file_path)?;
        if metadata.len() == 0 {
            return Ok(());
        }

        let file = File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let mut start = true;

        use std::io::BufRead;

        for line in reader.lines() {
            if let Ok(content) = line {
                if start {
                    start = false;
                    println!("{placeholder}");
                }
                println!("{}", content);
            } else if let Err(err) = line {
                eprintln!("Error reading line: {}", err);
            }
        }

        Ok(())
    }

    let stdout_path = get_stdout_path_cross_platform();
    read_and_print_file(&stdout_path, "STDOUT>")?;

    let stderr_path = get_stderr_path_cross_platform();
    read_and_print_file(&stderr_path, "STDERR>")?;

    Ok(())
}
