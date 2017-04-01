use xi_unicode::LineBreakIterator;

use super::*;

use self::Generate;

pub struct Xi
{
	pub priority: usize
}



impl Generate for Xi
{
	fn opportunities( &self, text: &str ) -> Vec< SplitPoint >
	{
		let mut result = LineBreakIterator::new( text ).map( |(byte_offset, hard)|
		{
			println!("break from Xi: {:?}", byte_offset );
			let mut start = byte_offset;

			// This only works if whitespace are bytes that are never composed in grapheme clusters.
			// TODO: verify if this holds
			//
			for ( i, _ ) in text[ ..byte_offset ].char_indices().rev().take_while( | &( _, c ) | util::char_is_whitespace( &c ) ).last()
			{
				start = i;
			}

			let mut s = SplitPoint::new( start, byte_offset, self.priority );
			s.mandatory = hard;
			s
		})

		.collect::< Vec<_> >();

		// xi adds a SplitPoint at the end of the string, and we don't want that. It's not useful for a wrap algorithm and might cause
		// wrap to return empty strings at the end, so we remove it.
		//
		// assert_eq!( ByteOffset( text.len() ), result.last().unwrap().start );
		assert_eq!( ByteOffset( text.len() ), result.last().unwrap().end   );

		result.pop();


		// The unicode annex 14 section 5 says that tab is to be considered like space when it comes to line breaking, however
		// the behaviour of Xi is not entirely the same when tabs and spaces are mixed:
		//
		// "foo\tbar"      gives break opportunity on offset 4, and 7
		// "foo   bar"     gives break opportunity on offset 6 and 9
		// "foo\t\t\tbar"  gives break opportunity on offset 6 and 9
		// "foo \t bar"    gives break opportunity on offset 4, 6 and 9
		// "foo\t \tbar"   gives break opportunity on offset 5, 6 and 9
		//
		// This means that for "foo \t bar", we will get overlaping split points, start: 3, end: 4 and start: 3, end 6.
		// In principle, this does not lead to wrong splitting in the wrapping algorithm, so for now we don't correct this.

		result
	}
}



#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn basic()
	{
		let s = "foo bar";

		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( &s )

			, vec![ SplitPoint::new( 3, 4, 0 ) ]
		);
	}


	#[test]
	fn more_spaces()
	{
		let s = "foo   bar";

		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( &s )

			, vec![ SplitPoint::new( 3, 6, 0 ) ]
		);
	}


	#[test]
	fn no_break_space()
	{
		let s = "foo\u{A0}bar";

		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( &s )

			, vec![]
		);
	}


	#[test]
	fn tabstop()
	{
		let s = "foo\tbar";

		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( &s )

			, vec![ SplitPoint::new( 3, 4, 0 ) ]
		);
	}


	#[test]
	fn hyphens()
	{
		// \u{ad} is Unicode U+00AD SOFT HYPHEN
		//
		let s = "co\u{ad}ca-co‧la";

		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( &s )

			, vec!
			  [
				  SplitPoint::new(  4,  4, 0 ) ,
				  SplitPoint::new(  7,  7, 0 ) ,
				  SplitPoint::new( 12, 12, 0 ) ,
			  ]
		);
	}


	// I don't know if this is really desirable behaviour, but in the worst case splits like this can be prevented with a filter.
	//
	#[test]
	fn hyphen_series()
	{
		let s = "bin--doo";

		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( &s )

			, vec![ SplitPoint::new( 5, 5, 0 ) ]
		);
	}


	// #[test]
	// fn tabstop2()
	// {
	// 	let s = "foo \t bar";

	// 	assert_eq!
	// 	(
	// 		  Xi{ priority: 0 }.opportunities( &s )

	// 		  , vec!
	// 		    [
	// 		  	    SplitPoint { start: ByteOffset( 3 ), end: ByteOffset( 6 ), glue: "", mandatory: false, priority: 0, width: None }
	// 		    ]
	// 	);
	// }
}
