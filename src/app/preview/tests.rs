use super::*;

fn card(tab: u64) -> Open {
	Open {
		target: Target::Tab(tab),
		card: Card {
			title: vec!["notes.md".into()],
			detail: "/tmp/notes.md".into(),
			body: true,
		},
	}
}

#[test]
fn a_held_hover_opens_only_after_the_delay() {
	let mut preview = Preview::default();
	let now = Instant::now();
	assert!(!preview.tracking());
	assert_eq!(preview.due(now + SHOW_DELAY), None);

	assert!(!preview.hover(Some(Target::Tab(7)), now));
	assert!(preview.tracking());
	assert_eq!(preview.deadline(), Some(now + SHOW_DELAY));
	assert_eq!(preview.due(now), None);
	assert_eq!(
		preview.due(now + SHOW_DELAY - Duration::from_millis(1)),
		None
	);
	assert_eq!(preview.due(now + SHOW_DELAY), Some(Target::Tab(7)));

	preview.show(card(7));
	assert_eq!(preview.due(now + Duration::from_secs(1)), None);
	assert_eq!(preview.deadline(), None);
	assert_eq!(preview.open().map(|open| open.target), Some(Target::Tab(7)));
}

#[test]
fn crossing_targets_restarts_the_delay_and_clears_the_old_card() {
	let mut preview = Preview::default();
	let now = Instant::now();
	preview.hover(Some(Target::Tab(1)), now);
	preview.show(card(1));
	assert!(preview.open().is_some());

	// Moving to another target closes the card at once and holds the new delay.
	assert!(
		preview.hover(Some(Target::Tab(2)), now + Duration::from_millis(20))
	);
	assert!(preview.open().is_none());
	let due = now + Duration::from_millis(20) + SHOW_DELAY;
	assert_eq!(preview.due(due - Duration::from_millis(1)), None);
	assert_eq!(preview.due(due), Some(Target::Tab(2)));

	// Motion inside the same target keeps the original deadline.
	preview.hover(Some(Target::Tab(2)), now + Duration::from_millis(60));
	assert_eq!(preview.deadline(), Some(due));
}

#[test]
fn leaving_every_target_forgets_the_hover() {
	let mut preview = Preview::default();
	let now = Instant::now();
	preview.hover(Some(Target::Tab(3)), now);
	assert!(!preview.hover(None, now + Duration::from_millis(10)));
	assert!(!preview.tracking());
	assert_eq!(preview.deadline(), None);

	preview.hover(Some(Target::Tab(3)), now);
	preview.show(card(3));
	assert!(preview.hover(None, now + Duration::from_millis(20)));
	assert!(preview.open().is_none());
	assert!(!preview.tracking());
}

#[test]
fn the_chrome_body_rect_belongs_to_the_open_card() {
	let mut preview = Preview::default();
	let rect = Rect {
		x: 10.0,
		y: 40.0,
		w: 240.0,
		h: 150.0,
	};
	preview.set_body(Some(rect));
	assert_eq!(preview.body(), None);
	preview.hover(Some(Target::Tab(1)), Instant::now());
	preview.show(card(1));
	assert_eq!(preview.body(), None);
	preview.set_body(Some(rect));
	assert_eq!(preview.body(), Some(rect));
	preview.dismiss();
	assert_eq!(preview.body(), None);
}

/// The application-level flow: a hovering pointer arms the delay, the tick
/// opens the card, and motion away closes it.
#[test]
fn a_resting_pointer_opens_the_card_after_the_delay() {
	use crate::{
		app::{Event, SendEvent},
		cli::{LaunchOptions, Mode},
	};
	use std::sync::mpsc;

	#[derive(Clone)]
	struct Proxy(mpsc::Sender<Event>);
	impl SendEvent for Proxy {
		fn try_send(&self, event: Event) -> bool {
			self.0.send(event).is_ok()
		}
	}

	let (tx, _events) = mpsc::channel();
	let mut app = App::new(
		LaunchOptions {
			mode: Mode::Smoke,
			offline: true,
			width: 800,
			height: 600,
			options: crate::test_support::options(),
			..Default::default()
		},
		Proxy(tx),
	);
	app.preferences.values = Default::default();
	let second = std::path::PathBuf::from("/tmp/second.md");
	app.readers
		.open(std::path::PathBuf::from("/tmp/first.md"), Instant::now());
	app.readers.open(second.clone(), Instant::now());
	let layout = app.tab_layout();
	let cursor = (layout.rects[0].x + 20.0, 20.0);
	app.interaction.cursor = cursor;
	let now = Instant::now();

	assert!(!app.hover_preview(now));
	assert!(app.preview.open().is_none());
	app.preview_tick(now + Duration::from_millis(50));
	assert!(app.preview.open().is_none());

	app.preview_tick(now + SHOW_DELAY);
	let open = app.preview.open().expect("the card did not open");
	assert_eq!(open.target, Target::Tab(app.readers.entries()[0].id));
	assert_eq!(open.card.title, ["first.md"]);
	assert!(open.card.detail.ends_with("first.md"));
	assert!(!open.card.body, "an unrendered tab must not reserve a body");

	// The chrome paints a card, and the preview never opens inside a panel.
	let draws = app.overlay();
	assert!(draws.iter().any(|draw| matches!(
		draw,
		crate::layout::Draw::Box { radius, .. } if *radius > 0.0
	)));
	app.interaction.panel = crate::state::PanelPage::Tabs;
	app.preview_tick(now + Duration::from_secs(1));
	assert!(app.preview.open().is_none());
	app.interaction.panel = crate::state::PanelPage::Closed;

	// Opening again and moving off the strip closes the card at once.
	app.interaction.cursor = cursor;
	app.hover_preview(now + Duration::from_secs(2));
	let at = now + Duration::from_secs(2) + SHOW_DELAY;
	app.preview_tick(at);
	assert!(app.preview.open().is_some());
	app.interaction.cursor = (400.0, 300.0);
	assert!(app.hover_preview(at + Duration::from_millis(1)));
	assert!(app.preview.open().is_none());
}
