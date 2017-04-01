pub mod french;

pub use super::*;

pub trait Filter
{
	fn run( &self, text: &str, splits: &mut Vec<SplitPoint> );
}
