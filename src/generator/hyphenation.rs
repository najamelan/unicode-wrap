use hyphenation_crate::{FullTextHyphenation, Corpus};
use super::*;

use self::Generate;

pub struct Hyphenator<'a, 'b>
{
	pub priority: usize      ,
	pub corpus  : &'a Corpus ,
	pub glue    : &'b str    ,
}



impl<'a, 'b> Generate for Hyphenator<'a, 'b>
{
	fn opportunities( &self, text: &str ) -> Vec< SplitPoint >
	{
		let list = text.fulltext_opportunities( self.corpus );

		let result = list.iter().map( |byte_offset|
		{
			println!("break from Hyphenator: {:?}", byte_offset );

			SplitPoint { start: ByteOffset( *byte_offset ), end: ByteOffset( *byte_offset ), glue: self.glue, mandatory: false, width: None, priority: self.priority }
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

		assert_eq!
		(
			  Hyphenator{ priority: 0, glue: "-", corpus: &c }.opportunities( &s )

			, vec!
			  [
				  SplitPoint { start: ByteOffset(2), end: ByteOffset(2), glue: "-", mandatory: false, priority: 0, width: None } ,
				  SplitPoint { start: ByteOffset(6), end: ByteOffset(6), glue: "-", mandatory: false, priority: 0, width: None } ,
			  ]
		);
	}

	#[test]
	fn should_not_return_splitpoints_for_spaces()
	{
		let s = "hyphe nation";
		let c = hyphenation_crate::load( Language::English_US ).unwrap();

		assert_eq!
		(
			  Hyphenator{ priority: 0, glue: "-", corpus: &c }.opportunities( &s )

			, vec!
			  [
				  SplitPoint { start: ByteOffset(2), end: ByteOffset(2), glue: "-", mandatory: false, priority: 0, width: None } ,
				  SplitPoint { start: ByteOffset(8), end: ByteOffset(8), glue: "-", mandatory: false, priority: 0, width: None } ,
			  ]
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
			  Hyphenator{ priority: 0, glue: "-", corpus: &c }.opportunities( &s )

			, vec![]
		);
	}
}
