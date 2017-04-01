use xi_unicode::LineBreakIterator;

use super::*;

use self::interface::Generator;

pub struct Xi
{
	pub priority: usize
}



impl Generator for Xi
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

			SplitPoint { start: ByteOffset( start ), end: ByteOffset( byte_offset ), glue: "", mandatory: hard, width: None, priority: self.priority }
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

			, vec!
			  [
				    SplitPoint { start: ByteOffset( 3 ), end: ByteOffset( 4 ), glue: "", mandatory: false, priority: 0, width: None }
			  ]
		);
	}


	#[test]
	fn more_spaces()
	{
		let s = "foo   bar";

		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( &s )

			, vec!
			  [
				    SplitPoint { start: ByteOffset( 3 ), end: ByteOffset( 6 ), glue: "", mandatory: false, priority: 0, width: None }
			  ]
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

			  , vec!
			    [
			  	    SplitPoint { start: ByteOffset( 3 ), end: ByteOffset( 4 ), glue: "", mandatory: false, priority: 0, width: None }
			    ]
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
