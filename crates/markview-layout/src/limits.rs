//! Work and byte allowances for document layout.
//!
//! The defaults are chosen so ordinary documents never reach them: a limit is
//! a guard against pathological input, not a policy about how much a user may
//! read. See `docs/developers/security.md` for the threat model.

/// Iteration and byte budgets for document layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Limits {
	/// Total candidate line breaks the knapsack pass and its greedy fallback
	/// may evaluate for one paragraph.
	pub linebreak_evaluations: usize,
	/// A single source line longer than this is rendered without syntax
	/// colors instead of being handed to a regex highlighter.
	pub highlight_line_bytes: usize,
	/// Total code bytes highlighted per layout pass; the remainder is shown
	/// uncolored.
	pub highlight_bytes: usize,
	/// A single formula larger than this is reported as a math error.
	pub math_formula_bytes: usize,
	/// Total formula bytes laid out per layout pass.
	pub math_bytes: usize,
	/// Columns retained from a document table.
	pub table_columns: usize,
	/// Rows retained from a document table.
	pub table_rows: usize,
	/// Cells retained from a document table, which caps a wide and long table
	/// at once.
	pub table_cells: usize,
}

impl Default for Limits {
	fn default() -> Self {
		Self {
			linebreak_evaluations: 2_000_000,
			highlight_line_bytes: 64 * 1024,
			highlight_bytes: 16 * 1024 * 1024,
			math_formula_bytes: 256 * 1024,
			math_bytes: 8 * 1024 * 1024,
			table_columns: 256,
			table_rows: 16_384,
			table_cells: 131_072,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn every_budget_is_nonzero_and_generous() {
		let l = Limits::default();
		assert!(l.linebreak_evaluations >= 1_000_000);
		// A 10K-character formula, even in three-byte characters, must fit.
		assert!(l.math_formula_bytes >= 10_000 * 3);
		assert!(l.math_bytes > l.math_formula_bytes);
		assert!(l.table_columns >= 64 && l.table_rows >= 1024);
		assert!(l.table_cells >= l.table_columns);
	}
}
