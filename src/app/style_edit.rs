//! The style priority list shared by Settings and Export.
pub enum Operation {
	Toggle,
	Up,
	Down,
}
pub struct Edit {
	pub id: String,
	pub operation: Operation,
	pub valid: bool,
}
impl Edit {
	pub(super) fn apply(self, ids: &mut Vec<String>) -> bool {
		let position = ids.iter().position(|id| id == &self.id);
		match self.operation {
			Operation::Toggle if position.is_some() => {
				ids.retain(|id| id != &self.id)
			}
			Operation::Toggle if self.valid => ids.insert(0, self.id),
			Operation::Up if position.is_some_and(|i| i > 0) => {
				let i = position.unwrap();
				ids.swap(i, i - 1);
			}
			Operation::Down if position.is_some_and(|i| i + 1 < ids.len()) => {
				let i = position.unwrap();
				ids.swap(i, i + 1);
			}
			_ => return false,
		}
		true
	}
}
