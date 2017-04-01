pub mod unicode_standard;

pub use super::*;

pub trait Generate
{
	fn opportunities( &self, text: &str ) -> Vec< SplitPoint >;
}
