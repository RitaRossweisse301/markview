//! Budgets for converting CommonMark and embedded HTML into document nodes.
#[derive(Clone, Copy)]
pub(crate) struct Limits {
	/// Maximum inline AST recursion. Comrak builds the AST iteratively, so
	/// this bounds only Markview's own recursion, which needs about 0.5 KB of
	/// stack per level.
	pub inline_depth: usize,
	/// Maximum block nesting retained as structure; deeper content is kept as
	/// source text.
	pub block_depth: usize,
	/// Maximum raw HTML scopes open at once. Inline HTML tags are AST
	/// siblings, so an unclosed run of them grows the scope stack without
	/// consuming `inline_depth`; past this many the tags stay source text.
	/// Far above any document's real nesting, because an unclosed run is what
	/// this guards against, not depth.
	pub html_scopes: usize,
}
impl Default for Limits {
	fn default() -> Self {
		Self {
			inline_depth: 256,
			block_depth: 256,
			html_scopes: 8_192,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parsing_budgets_are_nonzero_and_generous() {
		let limits = Limits::default();
		assert_eq!(limits.inline_depth, 256);
		assert_eq!(limits.block_depth, 256);
		assert!(limits.html_scopes >= 4_096);
	}
}
