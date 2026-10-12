//! RGBA color values carried by semantic text and appearance.
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color(pub u32);
impl Color {
	pub fn rgba(self) -> [f32; 4] {
		[
			((self.0 >> 24) & 255) as f32 / 255.,
			((self.0 >> 16) & 255) as f32 / 255.,
			((self.0 >> 8) & 255) as f32 / 255.,
			(self.0 & 255) as f32 / 255.,
		]
	}
}
impl<'de> Deserialize<'de> for Color {
	fn deserialize<D: serde::Deserializer<'de>>(
		d: D,
	) -> std::result::Result<Self, D::Error> {
		let s = String::deserialize(d)?;
		let h = s.strip_prefix('#').ok_or_else(|| {
			serde::de::Error::custom("expected #RRGGBB or #RRGGBBAA")
		})?;
		if !matches!(h.len(), 6 | 8)
			|| !h.bytes().all(|c| c.is_ascii_hexdigit())
		{
			return Err(serde::de::Error::custom(
				"expected #RRGGBB or #RRGGBBAA",
			));
		}
		let v = u32::from_str_radix(h, 16).map_err(serde::de::Error::custom)?;
		Ok(Self(if h.len() == 6 { v << 8 | 255 } else { v }))
	}
}
