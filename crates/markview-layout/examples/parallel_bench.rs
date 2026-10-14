//! Geometry-only measurement; run each thread count in a fresh release process.
use markview_layout::{
	background::Direct,
	fonts::FontConfig,
	layout::{LayoutEngine, LayoutOptions},
	shaping::TextShaper,
};
use std::{path::Path, sync::Arc, time::Instant};

fn rss(field: &str) -> u64 {
	std::fs::read_to_string("/proc/self/status")
		.unwrap_or_default()
		.lines()
		.find_map(|line| line.strip_prefix(field))
		.and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
		.unwrap_or_default()
		* 1024
}

fn main() -> anyhow::Result<()> {
	let args: Vec<_> = std::env::args().collect();
	let source = std::fs::read_to_string(&args[1])?;
	let threads = args[2].parse()?;
	let iterations = args[3].parse()?;
	let document = markview_commonmark::parse(source);
	let options = LayoutOptions {
		fonts: FontConfig {
			ignore_system_fonts: true,
			directories: vec![
				Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fonts"),
			],
			..Default::default()
		},
		..Default::default()
	};
	TextShaper::warm_fonts(&options.fonts);
	let mut engine =
		LayoutEngine::with_executor(Arc::new(Direct), Arc::new(|| {}));
	engine.set_parallelism(threads)?;
	engine.set_parallel_start(1098.);
	println!(
		"threads,iteration,phase,viewport_ms,complete_ms,reused,rss_bytes,peak_bytes"
	);
	for iteration in 0..iterations {
		for phase in ["full", "cached"] {
			if phase == "full" {
				engine.clear_document_cache();
			}
			let start = Instant::now();
			let mut viewport = None;
			let snapshot = engine
				.layout_progressive(
					&document,
					&options,
					&Default::default(),
					|prefix| {
						if prefix.height >= 1098. && viewport.is_none() {
							viewport = Some(start.elapsed());
						}
						true
					},
				)
				.unwrap();
			let complete = start.elapsed();
			println!(
				"{threads},{iteration},{phase},{:.4},{:.4},{},{},{}",
				viewport.unwrap_or(complete).as_secs_f64() * 1000.,
				complete.as_secs_f64() * 1000.,
				snapshot.reused,
				rss("VmRSS:"),
				rss("VmHWM:")
			);
		}
	}
	Ok(())
}
