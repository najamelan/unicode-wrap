use super::super::SplitPoint;

pub trait Generator
{
	fn opportunities( &self, text: &str ) -> Vec< SplitPoint >;
}
