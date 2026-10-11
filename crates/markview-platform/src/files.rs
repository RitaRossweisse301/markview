//! Native file access and permissions; callers validate contents and own transactions.
use std::{fs, io, path::Path};
#[cfg(unix)]
pub fn open_nonblocking(path: &Path) -> io::Result<fs::File> {
	use std::os::unix::fs::OpenOptionsExt;
	// `O_NONBLOCK` has no effect on a regular file; it only decides whether the
	// open waits. Standard Rust opens already set `O_CLOEXEC`, so no descriptor
	// escapes into a child.
	fs::OpenOptions::new()
		.read(true)
		.custom_flags(libc::O_NONBLOCK)
		.open(path)
}

#[cfg(not(unix))]
pub fn open_nonblocking(path: &Path) -> io::Result<fs::File> {
	// Windows has no portable non-blocking open, so opening a device or a pipe
	// can still wait there; the type check still refuses it once it returns.
	fs::File::open(path)
}

pub fn private_file_options(options: &mut fs::OpenOptions) {
	#[cfg(unix)]
	{
		use std::os::unix::fs::OpenOptionsExt;
		options.mode(0o600);
	}
	#[cfg(not(unix))]
	let _ = options;
}

pub fn export_tempfile(parent: &Path) -> io::Result<tempfile::NamedTempFile> {
	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		tempfile::Builder::new()
			.permissions(fs::Permissions::from_mode(0o666))
			.tempfile_in(parent)
	}
	#[cfg(not(unix))]
	{
		tempfile::NamedTempFile::new_in(parent)
	}
}

pub fn config_base() -> Option<std::path::PathBuf> {
	#[cfg(not(target_os = "android"))]
	use std::path::PathBuf;
	#[cfg(target_os = "android")]
	{
		crate::android::data_path()
	}
	#[cfg(target_os = "windows")]
	{
		std::env::var_os("APPDATA").map(PathBuf::from)
	}
	#[cfg(target_os = "macos")]
	{
		std::env::var_os("HOME")
			.map(|p| PathBuf::from(p).join("Library/Application Support"))
	}
	#[cfg(not(any(
		target_os = "windows",
		target_os = "macos",
		target_os = "android"
	)))]
	{
		std::env::var_os("XDG_CONFIG_HOME")
			.map(PathBuf::from)
			.filter(|p| p.is_absolute())
			.or_else(|| {
				std::env::var_os("HOME")
					.map(|p| PathBuf::from(p).join(".config"))
			})
	}
}
pub fn system_locale() -> Option<String> {
	sys_locale::get_locale()
}
