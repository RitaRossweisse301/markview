//! Semantic image sources, before resource loading or layout.
/// Prefix that marks an image source as a Mermaid diagram rather than a path
/// or URL. The scheduler strips it and renders the remainder as diagram source.
pub const MERMAID_SCHEME: &str = "mermaid:";

/// One Markdown or HTML image as the document sees it, before any I/O.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImageSpec {
	pub src: String,
	pub alt: String,
	pub title: String,
	/// Explicit `width` / `height` attributes; `None` keeps the aspect ratio.
	pub width: Option<u32>,
	pub height: Option<u32>,
}

/// The image source for a Mermaid fence's code, so a diagram travels through
/// the image scheduler like any other source.
pub fn mermaid_source(code: &str) -> String {
	format!("{MERMAID_SCHEME}{code}")
}
