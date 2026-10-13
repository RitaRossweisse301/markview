//! CPU-only shaping measurements with pinned fonts and warm font caches.
use markview_layout::{
	fonts::FontConfig,
	layout::{LayoutEngine, LayoutOptions},
	profile,
	scene::Paint,
	shaping::TextShaper,
};
use std::{hint::black_box, path::Path, time::Instant};

fn fonts() -> FontConfig {
	FontConfig {
		ignore_system_fonts: true,
		directories: vec![
			Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fonts"),
		],
		..Default::default()
	}
}

fn measure(name: &str, iterations: usize, mut run: impl FnMut()) {
	for _ in 0..10 {
		run();
	}
	profile::reset();
	let start = Instant::now();
	for _ in 0..iterations {
		run();
	}
	let elapsed = start.elapsed().as_secs_f64() * 1e6 / iterations as f64;
	let stages = profile::totals();
	eprintln!(
		"{name}: total_us={elapsed:.3} font_choose_us={:.3} shape_build_us={:.3}",
		stages["layout.font_choose_ms"] * 1000. / iterations as f64,
		stages["layout.shape_build_ms"] * 1000. / iterations as f64,
	);
}

#[test]
#[ignore = "release-mode shaping and layout timing report"]
fn shaping_measurement() {
	profile::enable();
	for (name, text, iterations) in [
		("short", "Open settings.toml".to_owned(), 10000),
		(
			"latin",
			"A readable file with office typography. ".repeat(32),
			1000,
		),
		(
			"cjk",
			"中文阅读与英文 typography 混合排版。".repeat(32),
			1000,
		),
	] {
		let mut shaper = TextShaper::with_fonts(fonts());
		measure(&format!("width-{name}"), iterations, || {
			black_box(shaper.text_width(black_box(&text), 18.));
		});
		measure(&format!("glyphs-{name}"), iterations, || {
			black_box(shaper.label_measured(
				black_box(&text),
				18.,
				0.,
				0.,
				Paint::Text,
			));
		});
	}
	for (name, source) in [
		(
			"plain",
			"A readable file with office typography. ".repeat(128),
		),
		(
			"styled",
			"plain **bold** and *italic* 中文文字 ".repeat(128),
		),
	] {
		let document = markview_commonmark::parse(source);
		let options = LayoutOptions {
			fonts: fonts(),
			..Default::default()
		};
		let mut engine = LayoutEngine::new();
		measure(&format!("layout-{name}"), 40, || {
			engine.clear_document_cache();
			black_box(engine.layout(black_box(&document), &options));
		});
	}
}
