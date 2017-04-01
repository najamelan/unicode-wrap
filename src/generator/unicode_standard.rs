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
		assert_eq!( ByteOffset( text.len() ), result.last().unwrap().start );
		assert_eq!( ByteOffset( text.len() ), result.last().unwrap().end   );

		result.pop();

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
				    SplitPoint { start: ByteOffset( 3       ), end: ByteOffset( 4       ), glue: "", mandatory: false, priority: 0, width: None }
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
				    SplitPoint { start: ByteOffset( 3       ), end: ByteOffset( 6       ), glue: "", mandatory: false, priority: 0, width: None }
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

			, vec!
			  [
			  ]
		);
	}
}
