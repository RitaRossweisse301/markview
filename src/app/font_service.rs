//! Host execution of the Fonts component's effects.
use super::{
	App, Event,
	font_panel::{Effect, Message},
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Downloads {
	runs: HashMap<u64, HashMap<String, tokio_util::sync::CancellationToken>>,
}

impl<P: super::SendEvent> App<P> {
	pub(super) fn font_message(&mut self, message: Message) {
		self.dispatch_ui(super::ui_model::Message::Fonts(message));
	}
	pub(super) fn font_effect(&mut self, effect: Effect) {
		match effect {
			Effect::Download { run, families } => {
				let ids: Vec<_> =
					families.iter().map(|family| family.id.clone()).collect();
				let Some(dir) = crate::fonts::directory() else {
					self.font_message(Message::Failed {
						run,
						ids,
						note: "No user configuration directory".into(),
					});
					return;
				};
				let services = self.services.handle.clone();
				let cancels: HashMap<_, _> = ids
					.iter()
					.map(|id| (id.clone(), services.cancel.child_token()))
					.collect();
				self.font_downloads.runs.insert(run, cancels.clone());
				let proxy = self.proxy.clone();
				let handle = services.clone();
				let progress_proxy = proxy.clone();
				if !services.submit(async move {
					let transport = crate::net::Downloader::new("Font");
					let summary = crate::fonts::run_async(
						&families,
						&dir,
						&transport,
						&handle,
						cancels,
						move |progress| {
							progress_proxy.send(Event::Fonts(
								Message::Progress {
									run,
									progress: Box::new(progress),
								},
							))
						},
					)
					.await;
					proxy.send(Event::Fonts(Message::Settled {
						run,
						summary: Box::new(summary),
					}));
				}) {
					self.font_downloads.runs.remove(&run);
					self.font_message(Message::Failed {
						run,
						ids,
						note: "Font service closed".into(),
					});
				}
			}
			Effect::Cancel(id) => {
				for cancels in self.font_downloads.runs.values() {
					if let Some(cancel) = cancels.get(&id) {
						cancel.cancel();
					}
				}
			}
			Effect::OpenFolder => {
				let result = crate::fonts::directory()
					.ok_or_else(|| {
						anyhow::anyhow!("No user configuration directory")
					})
					.and_then(|dir| {
						std::fs::create_dir_all(&dir)?;
						markview_platform::open_external(dir)?;
						Ok(())
					});
				if let Err(error) = result {
					self.font_message(Message::Note(format!("{error:#}")));
				}
			}
			Effect::Offline => self.notify(
				self.preferences.values.lang().status_offline(),
				true,
				4,
			),
			Effect::Completed {
				run,
				stored,
				failure,
			} => {
				self.font_downloads.runs.remove(&run);
				self.register_fonts(stored);
				self.settings_resources.invalidate();
				self.refresh_settings_resources(false);
				if let Some(reason) = failure {
					self.notify(&reason, true, 6);
				}
			}
		}
	}
}
