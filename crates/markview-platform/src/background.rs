//! Native process detachment for the desktop launcher.
use std::process::{Command, Stdio};
pub fn background_allowed() -> bool {
	if !crate::desktop_cli() {
		return false;
	}
	#[cfg(target_os = "macos")]
	{
		// SAFETY: `getppid` takes no arguments and accesses no Rust memory.
		#[allow(unsafe_code)]
		if unsafe { libc::getppid() } == 1 {
			return false;
		}
	}
	true
}
pub fn detach(child: &mut Command) {
	child
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null());
	#[cfg(unix)]
	{
		use std::os::unix::process::CommandExt;
		// SAFETY: The hook only calls async-signal-safe `setsid` and reads
		// `errno`; it allocates nothing and touches no shared Rust state.
		#[allow(unsafe_code)]
		unsafe {
			child.pre_exec(|| {
				if libc::setsid() == -1 {
					return Err(std::io::Error::last_os_error());
				}
				Ok(())
			});
		}
	}
	#[cfg(windows)]
	{
		use std::os::windows::process::CommandExt;
		use windows_sys::Win32::System::Threading::{
			CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS,
		};
		child.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
	}
}
