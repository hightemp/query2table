//! Running the external tools.

use std::io;
use std::path::Path;
use std::process::{Output, Stdio};
use std::time::Duration;

use tokio::process::{Child, Command};

const SPAWN_ATTEMPTS: u32 = 6;

/// A command for an external program that opens no console window on Windows and is killed
/// when dropped (a stopped run).
pub fn command(program: &Path) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.kill_on_drop(true);
    cmd
}

/// Spawns `cmd`, retrying on "Text file busy": Linux reports it for a program that was just
/// written while another thread was forking (an update during a run).
pub async fn spawn(cmd: &mut Command) -> io::Result<Child> {
    let mut delay = Duration::from_millis(10);
    for _ in 1..SPAWN_ATTEMPTS {
        match cmd.spawn() {
            Err(e) if e.kind() == io::ErrorKind::ExecutableFileBusy => {
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            result => return result,
        }
    }
    cmd.spawn()
}

/// Like `Command::output`, with the retry of [`spawn`].
pub async fn output(cmd: &mut Command) -> io::Result<Output> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    spawn(cmd).await?.wait_with_output().await
}
