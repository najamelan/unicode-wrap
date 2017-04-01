pub mod unicode_width;

pub use super::*;

pub trait TextWidth
{
	fn measure( &self, text: &str ) -> usize;
}

