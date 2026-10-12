use markview_layout::{
	background::Direct,
	fonts::FontConfig,
	layout::{LayoutEngine, LayoutOptions},
	style::Stylesheet,
};
use markview_render::{Renderer, Theme, View};
use std::{collections::HashMap, sync::Arc, time::Duration};

fn fonts() -> FontConfig {
	FontConfig::from_faces(
		0x72656e646572,
		[
			include_bytes!(
				"../../markview-layout/tests/fonts/NotoSerif-Regular-subset.otf"
			)
			.as_slice(),
			include_bytes!(
				"../../markview-layout/tests/fonts/NotoSans-Bold-subset.otf"
			)
			.as_slice(),
			include_bytes!(
				"../../markview-layout/tests/fonts/NotoSansMono-Regular-subset.otf"
			)
			.as_slice(),
		]
		.into_iter()
		.map(|data| parley::fontique::Blob::new(Arc::new(data)))
		.collect(),
	)
}

fn view<'a>(
	theme: Theme,
	scale: f32,
	horizontal: &'a HashMap<(usize, usize), f32>,
) -> View<'a> {
	View {
		width: (400.0 * scale) as u32,
		height: (500.0 * scale) as u32,
		scale,
		left: 20.0,
		top: 20.0,
		bottom: 20.0,
		scroll: 0.0,
		theme,
		horizontal,
		selection: None,
		revision: 1,
		hovered_link: None,
		hovered_overflow: None,
		held_overflow: None,
	}
}

#[test]
fn markdown_renders_visible_and_identical_cached_frames() -> anyhow::Result<()>
{
	let doc = markview_commonmark::parse(
		"# Rendering\n\nA paragraph with $x^2$.\n\n- first\n- second\n\n```rust\nlet answer = 42;\n```\n",
	);
	let mut engine =
		LayoutEngine::with_executor(Arc::new(Direct), Arc::new(|| {}));
	let mut renderer = pollster::block_on(Renderer::new(None))?;
	eprintln!("Render test adapter: {}", renderer.adapter_name);
	let horizontal = HashMap::new();
	for (theme, scale) in [(Theme::Light, 1.0), (Theme::Dark, 1.25)] {
		let sheet = Stylesheet::bundled(theme == Theme::Dark);
		renderer.set_stylesheet(sheet.clone());
		let options = LayoutOptions {
			width: 360.0,
			fonts: fonts(),
			stylesheet: sheet,
			..Default::default()
		};
		let snapshot = engine.layout(&doc, &options);
		let snapshot = if engine.wait_highlights() {
			engine.layout(&doc, &options)
		} else {
			snapshot
		};
		let view = view(theme, scale, &horizontal);
		let target = renderer.offscreen(view.width, view.height);
		let submission = renderer.render(
			&snapshot,
			&view,
			&[],
			&target.create_view(&Default::default()),
		)?;
		renderer.wait(Some(submission))?;
		let first = renderer.read_pixels(&target)?;
		assert_eq!((first.width, first.height), (view.width, view.height));
		let background = &first.rgba[..4];
		let visible = first
			.rgba
			.as_chunks::<4>()
			.0
			.iter()
			.filter(|p| p.as_slice() != background)
			.count();
		assert!(visible > 1000, "Markdown frame is empty: {visible} pixels");
		let cached = engine.layout(&doc, &options);
		assert_eq!(cached.reused, cached.blocks.len());
		let submission = renderer.render(
			&cached,
			&view,
			&[],
			&target.create_view(&Default::default()),
		)?;
		renderer.wait(Some(submission))?;
		assert_eq!(first.rgba, renderer.read_pixels(&target)?.rgba);
	}
	Ok(())
}

/// A prewarm pass prepares the screenful below, which the reader is not
/// looking at. It must still fill the atlases, and it must leave both the
/// frame and the paint order of the next one exactly as they were.
#[test]
fn prewarm_prepares_atlases_without_a_frame() -> anyhow::Result<()> {
	let source = (0..40)
		.map(|i| {
			format!(
				"# Heading {i}\n\nParagraph number {i} with $x^{{{i}}}$.\n\n"
			)
		})
		.collect::<String>();
	let doc = markview_commonmark::parse(source.as_str());
	let mut engine =
		LayoutEngine::with_executor(Arc::new(Direct), Arc::new(|| {}));
	let mut renderer = pollster::block_on(Renderer::new(None))?;
	let sheet = Stylesheet::bundled(false);
	renderer.set_stylesheet(sheet.clone());
	let options = LayoutOptions {
		width: 360.0,
		fonts: fonts(),
		stylesheet: sheet,
		..Default::default()
	};
	let snapshot = engine.layout(&doc, &options);
	let horizontal = HashMap::new();
	let view = view(Theme::Light, 1.0, &horizontal);
	let target = renderer.offscreen(view.width, view.height);
	let submission = renderer.render(
		&snapshot,
		&view,
		&[],
		&target.create_view(&Default::default()),
	)?;
	renderer.wait(Some(submission))?;
	let frame = renderer.read_pixels(&target)?.rgba;
	assert!(renderer.vertices() > 0, "the frame emitted no vertices");
	let rasterized = renderer.raster_stats().rasterized;
	renderer.prewarm(&snapshot, &view, Duration::from_millis(20));
	assert!(
		renderer.raster_stats().rasterized > rasterized,
		"the prewarm pass rasterized nothing"
	);
	assert_eq!(
		renderer.vertices(),
		0,
		"the prewarm pass emitted vertices for a frame it discards"
	);
	let submission = renderer.render(
		&snapshot,
		&view,
		&[],
		&target.create_view(&Default::default()),
	)?;
	renderer.wait(Some(submission))?;
	assert_eq!(frame, renderer.read_pixels(&target)?.rgba);
	Ok(())
}
