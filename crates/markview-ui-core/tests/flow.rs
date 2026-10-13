use markview_ui_core::{
	Component, Input, Interaction, Invalidation, Layout, Rect, Runtime, Update,
	Widget, WidgetId,
};

#[derive(Default)]
struct Page {
	text: String,
	revision: u64,
}
enum Message {
	Edit(&'static str),
	Complete(u64, &'static str),
}
impl Component for Page {
	type Message = Message;
	type Effect = (u64, String);
	type View<'a> = &'a str;
	fn view(&self) -> &str {
		&self.text
	}
	fn update(&mut self, message: Message) -> Update<Self::Effect> {
		match message {
			Message::Edit(text) => {
				self.revision += 1;
				self.text = text.into();
				Update::new(Invalidation::Layout)
					.effect((self.revision, self.text.clone()))
			}
			Message::Complete(revision, text) if revision == self.revision => {
				self.text = text.into();
				Update::new(Invalidation::Paint)
			}
			_ => Update::new(Invalidation::None),
		}
	}
}

#[test]
fn host_executes_effects_and_returns_versioned_completions() {
	let mut runtime = Runtime::new(Page::default());
	runtime.send(Message::Edit("first"));
	runtime.send(Message::Edit("second"));
	let update = runtime.flush();
	assert_eq!(update.effects, [(1, "first".into()), (2, "second".into())]);
	assert_eq!(update.invalidation, Invalidation::Layout);
	assert_eq!(runtime.view(), "second");
	runtime.send(Message::Complete(1, "stale"));
	assert_eq!(runtime.flush().invalidation, Invalidation::None);
	assert_eq!(runtime.view(), "second");
	runtime.send(Message::Complete(2, "ready"));
	assert_eq!(runtime.flush().invalidation, Invalidation::Paint);
	assert_eq!(runtime.view(), "ready");
	assert!(runtime.flush().effects.is_empty());
}

fn widget(id: u64, message: &str, x: f32) -> Widget<String> {
	Widget {
		id: WidgetId::new(0, id),
		rect: Rect {
			x,
			y: 0.0,
			w: 40.0,
			h: 30.0,
		},
		clip: None,
		enabled: true,
		focusable: true,
		message: message.into(),
	}
}

#[test]
fn capture_tracks_identity_across_message_and_geometry_changes() {
	let mut layout = Layout {
		widgets: vec![widget(1, "download", 0.0), widget(2, "download", 50.0)],
	};
	let mut state = Interaction::default();
	assert_eq!(state.handle(&layout, Input::Press((10.0, 10.0))), None);
	assert_eq!(state.captured, Some(WidgetId::new(0, 1)));
	state.handle(&layout, Input::Move((60.0, 10.0)));
	assert_eq!(state.captured, Some(WidgetId::new(0, 1)));
	assert_eq!(state.handle(&layout, Input::Release((60.0, 10.0))), None);
	state.handle(&layout, Input::Press((10.0, 10.0)));
	layout.widgets[0].message = "cancel".into();
	layout.widgets[0].rect.x = 5.0;
	assert_eq!(
		state.handle(&layout, Input::Release((10.0, 10.0))),
		Some("cancel".into())
	);
	assert_eq!(state.handle(&layout, Input::Release((10.0, 10.0))), None);
	state.handle(&layout, Input::Press((10.0, 10.0)));
	layout.widgets.remove(0);
	assert_eq!(state.handle(&layout, Input::Release((10.0, 10.0))), None);
	assert_eq!(state.captured, None);
}

#[test]
fn clipping_stacking_and_disabled_controls_share_one_layout() {
	let mut front = widget(1, "front", 0.0);
	front.clip = Some(Rect {
		x: 20.0,
		y: 0.0,
		w: 20.0,
		h: 30.0,
	});
	front.enabled = false;
	let layout = Layout {
		widgets: vec![front, widget(2, "back", 0.0)],
	};
	assert_eq!(layout.hit((10.0, 10.0)).unwrap().id, WidgetId::new(0, 2));
	assert_eq!(layout.hit((30.0, 10.0)).unwrap().id, WidgetId::new(0, 1));
	let mut state = Interaction::default();
	state.handle(&layout, Input::Press((30.0, 10.0)));
	assert_eq!(state.handle(&layout, Input::Release((30.0, 10.0))), None);
	state.handle(&layout, Input::Navigate { backward: false });
	assert_eq!(state.focus, Some(WidgetId::new(0, 2)));
	assert_eq!(state.handle(&layout, Input::Activate), Some("back".into()));
	state.handle(&layout, Input::Blur);
	assert_eq!(state.focus, None);
	assert_eq!(Layout::<Message>::default().next_focus(None, true), None);
}

#[test]
fn touch_expansion_keeps_clips_and_exact_hits_authoritative() {
	let mut clipped = widget(1, "clipped", 0.0);
	clipped.clip = Some(Rect {
		x: 20.0,
		y: 0.0,
		w: 20.0,
		h: 30.0,
	});
	let layout = Layout {
		widgets: vec![clipped, widget(2, "neighbor", 45.0)],
	};
	assert!(layout.hit_touch((10.0, 10.0), 44.0).is_none());
	assert_eq!(
		layout.hit_touch((44.0, 10.0), 44.0).unwrap().id,
		WidgetId::new(0, 2)
	);
	assert_eq!(
		layout.hit_touch((46.0, 10.0), 100.0).unwrap().id,
		WidgetId::new(0, 2)
	);
}
