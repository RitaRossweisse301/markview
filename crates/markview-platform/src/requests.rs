//! Activity-owned selection requests are released together at shutdown.
use std::path::PathBuf;

pub(crate) type Picked = Box<dyn FnOnce(Option<PathBuf>) + Send>;
#[derive(Default)]
pub(crate) struct Requests {
	pub picked: Option<Picked>,
	pub output: Option<tokio::sync::oneshot::Sender<Option<PathBuf>>>,
}
impl Requests {
	pub fn clear(&mut self) {
		self.picked.take();
		self.output.take();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::Arc;

	#[test]
	fn shutdown_releases_picker_capture_and_cancels_pending_output() {
		let capture = Arc::new(());
		let held = capture.clone();
		let (send, mut receive) = tokio::sync::oneshot::channel();
		let mut requests = Requests {
			picked: Some(Box::new(move |_| {
				let _ = &held;
				panic!(
					"shutdown must release the callback without invoking it"
				);
			})),
			output: Some(send),
		};
		assert_eq!(Arc::strong_count(&capture), 2);
		requests.clear();
		assert_eq!(Arc::strong_count(&capture), 1);
		assert_eq!(
			receive.try_recv(),
			Err(tokio::sync::oneshot::error::TryRecvError::Closed)
		);
		requests.clear();
	}
}
