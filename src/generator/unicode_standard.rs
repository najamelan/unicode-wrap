use xi_unicode::LineBreakIterator;

use super::*;

use self::interface::Generator;

pub struct Xi;



impl Generator for Xi
{
	fn opportunities( text: &str ) -> Vec< SplitPoint >
	{
		let result = LineBreakIterator::new( text ).map( |(byte_offset, hard)|
		{
			let mut start = byte_offset;

			// This only works if whitespace are bytes that are never composed in grapheme clusters.
			// TODO: verify if this holds
			//
			for ( i, _ ) in text[ ..byte_offset ].char_indices().rev().take_while( | &( _, c ) | util::char_is_whitespace( &c ) ).last()
			{
				start = i;
			}

			SplitPoint { start: ByteOffset( start ), end: ByteOffset( byte_offset ), glue: "", mandatory: hard }
		})

		.collect::< Vec<SplitPoint> >();

		// xi adds a SplitPoint at the end of the string, and we don't want that. It's not useful for a wrap algorithm, so
		// we remove it. Actually, for now, this split point will never serve because either the whole rest of the string
		// fits in a line, or if it doesn't this splitpoint wont fit either, so it will be ignored. So for now I comment this code.
		//
		// assert_eq!( ByteOffset( text.len() ), result.last().unwrap().span.start );
		// assert_eq!( ByteOffset( text.len() ), result.last().unwrap().span.end   );

		// result.pop();

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
			  Xi::opportunities( &s )

			, vec!
			  [
				    SplitPoint { start: ByteOffset( 3       ), end: ByteOffset( 4       ), glue: "", mandatory: false }
				  , SplitPoint { start: ByteOffset( s.len() ), end: ByteOffset( s.len() ), glue: "", mandatory: true  }
			  ]
		);
	}


	#[test]
	fn more_spaces()
	{
		let s = "foo   bar";

		assert_eq!
		(
			  Xi::opportunities( &s )

			, vec!
			  [
				    SplitPoint { start: ByteOffset( 3       ), end: ByteOffset( 6       ), glue: "", mandatory: false }
				  , SplitPoint { start: ByteOffset( s.len() ), end: ByteOffset( s.len() ), glue: "", mandatory: true  }
			  ]
		);
	}


	#[test]
	fn no_break_space()
	{
		let s = "foo\u{A0}bar";

		assert_eq!
		(
			  Xi::opportunities( &s )

			, vec!
			  [
				  SplitPoint { start: ByteOffset( s.len() ), end: ByteOffset( s.len() ), glue: "", mandatory: true  }
			  ]
		);
	}
}
