pub use super::*;

use self::Filter;

const NO_BREAK_BEFORE: &'static [char] = &
[
	'»'
];

const NO_BREAK_AFTER: &'static [char] = &
[
	'«'
];


#[ derive( PartialEq, Eq, Clone, Debug ) ]
//
pub struct French;

impl Filter for French
{
	fn run( &self, text: &str, splits: &mut Vec<SplitPoint> )
	{
		for split in splits
		{
			let c = text[ split.end.0.. ].chars().next();

			if c.is_some() && NO_BREAK_BEFORE.contains( &c.unwrap() )
			{
				split.enabled = false
			}


			let d = text[ ..split.start.0 ].chars().next_back();

			if d.is_some() && NO_BREAK_AFTER.contains( &d.unwrap() )
			{
				split.enabled = false
			}

		}
	}
}


#[cfg(test)]
mod tests
{
	use super::*;
	use generator::unicode_standard::Xi;

	fn filter( text: &str ) -> Vec< SplitPoint >
	{
		let mut opp = Xi{ priority: 0 }.opportunities( &text );

		French.run( &text, &mut opp );

		println!( "{:?}", opp );

		opp
	}


	#[test]
	fn dont_break_before()
	{
		let s = "a »";

		assert!( !filter( s )[ 0 ].enabled );
	}


	#[test]
	fn dont_break_after()
	{
		let s = "« a";

		assert!( !filter( s )[ 0 ].enabled );
	}


	#[test]
	fn dont_break_combined()
	{
		let s = "« a »";

		assert!( !filter( s )[ 0 ].enabled );
		assert!( !filter( s )[ 1 ].enabled );
	}

}
