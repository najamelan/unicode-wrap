use super::super::SplitPoint;

pub trait Generator
{
	fn opportunities( text: &str ) -> Vec< SplitPoint >;
}
