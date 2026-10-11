//! Release the shell before starting a desktop reader.
use crate::cli::{LaunchOptions, Mode};
use anyhow::{Context, Result};
use std::process::Command;

pub(super) fn launch(args: &LaunchOptions) -> Result<bool> {
	if args.mode != Mode::Window || args.foreground || args.background_child {
		return Ok(false);
	}
	if !markview_platform::background_allowed() {
		return Ok(false);
	}
	let mut child = Command::new(std::env::current_exe()?);
	child
		.arg("--background-child")
		.args(std::env::args_os().skip(1));
	markview_platform::detach(&mut child);
	child
		.spawn()
		.context("Cannot start the background reader")?;
	Ok(true)
}
