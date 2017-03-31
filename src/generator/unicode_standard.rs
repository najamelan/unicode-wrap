use xi_unicode::LineBreakIterator;

use super::interface::Generator;
use super::SplitPoint;
use super::ByteOffset;

pub struct Xi;

impl Generator for Xi
{
	fn opportunities( text: &str ) -> Vec< SplitPoint >
	{
		let result = LineBreakIterator::new( text ).map( |(byte_offset, hard)|
		{
			SplitPoint { span: ByteOffset( byte_offset )..ByteOffset( byte_offset ), glue: "", mandatory: hard }
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
		assert_eq!
		(
			  Xi::opportunities( "foo bar" )

			, vec!
			  [
				    SplitPoint { span: ByteOffset( 4 ) .. ByteOffset( 4 ), glue: "", mandatory: false }
				  , SplitPoint { span: ByteOffset( 7 ) .. ByteOffset( 7 ), glue: "", mandatory: true  }
			  ]
		);
	}
}
