use hyphenation_crate::{FullTextHyphenation, Corpus};
use super::*;

use self::Generate;


#[ derive( Clone, Debug ) ]
//
pub struct Hyphenator
{
	pub priority: usize  ,
	pub corpus  : Corpus ,
	pub glue    : String ,
}



impl Generate for Hyphenator
{
	fn opportunities( &self, text: &str ) -> Vec< SplitPoint >
	{
		let list = text.fulltext_opportunities( &self.corpus );

		let result = list.iter().map( |byte_offset|
		{
			if cfg!( debug_assertions ) { println!("break from Hyphenator: {:?}", byte_offset ) }

			let mut s = SplitPoint::new( *byte_offset, *byte_offset, self.priority );
			s.glue = self.glue.clone();
			s
		})

		.collect::< Vec<_> >();

		result
	}
}



#[cfg(test)]
mod tests
{
	use super::*;
	use hyphenation_crate::Language;

	#[test]
	fn basic()
	{
		let s = "hyphenation";
		let c = hyphenation_crate::load( Language::English_US ).unwrap();

		let mut s1 = SplitPoint::new( 2, 2, 0 ); s1.glue = "-".to_string();
		let mut s2 = SplitPoint::new( 6, 6, 0 ); s2.glue = "-".to_string();

		assert_eq!
		(
			  Hyphenator{ priority: 0, glue: "-".to_string(), corpus: c }.opportunities( &s )

			, vec![ s1, s2 ]
		);
	}

	#[test]
	fn should_not_return_splitpoints_for_spaces()
	{
		let s = "hyphe nation";
		let c = hyphenation_crate::load( Language::English_US ).unwrap();

		let mut s1 = SplitPoint::new( 2, 2, 0 ); s1.glue = "-".to_string();
		let mut s2 = SplitPoint::new( 8, 8, 0 ); s2.glue = "-".to_string();

		assert_eq!
		(
			  Hyphenator{ priority: 0, glue: "-".to_string(), corpus: c }.opportunities( &s )

			, vec![ s1, s2 ]
		);
	}

	#[test]
	fn should_not_return_existing_hyphens()
	{

		// \u{ad} is Unicode U+00AD SOFT HYPHEN
		//
		let s = "co\u{ad}ca-co‧la";
		let c = hyphenation_crate::load( Language::English_US ).unwrap();

		assert_eq!
		(
			  Hyphenator{ priority: 0, glue: "-".to_string(), corpus: c }.opportunities( &s )

			, vec![]
		);
	}
}
