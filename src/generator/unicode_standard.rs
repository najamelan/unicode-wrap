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
		LineBreakIterator::new( text ).map( |(byte_offset, hard)|
		{
			let mut start = byte_offset;
			let mut end   = byte_offset;
			let mut glue  = String::with_capacity( 2 );


			// Recreate the glue for hard breaks and set the start index to before the chars we move to glue. The glue will be set
			// to the actual line breaking character, so that we don't change existing differences between the existing protocols
			// and meaning of the different characters.
			// If conversion needs to be made, filters can manipulate the splitpoints later (eg. to convert between linux and windows line endings).
			//
			if hard  &&  end != text.len()
			{
				if let Some(( i , ch )) = text[ ..end ].char_indices().rev().nth( 0 )
				{
					start = i ;
					glue.insert( 0, ch );

					if ch == '\n'
					{
						if let Some(( i , ch )) = text[ ..i ].char_indices().rev().nth( 0 )
						{
							if ch == '\r' // cariage return
							{
								start = i;
								glue.insert( 0, ch );
							}
						}
					}
				}
			}


			else
			{
				// This only works if whitespace are bytes that are never composed in grapheme clusters.
				// TODO: verify if this holds
				//
				for ( i, _ ) in text[ ..end ].char_indices().rev().take_while( | &( _, c ) | util::char_is_whitespace( &c ) ).last()
				{
					start = i;
				}


				// The eot position
				//
				if hard
				{
					// There is no whitespace, this is just the end.
					//
					if start == end
					{
						glue = "".to_string();
					}

					// if the split opportunity didn't actually come from the spaces, but rather because it was the end of the string,
					// let's not consume them.
					//
					else
					{
						glue = "\n".to_string();
						end = start;
					}
				}

				else
				{
					glue = "\n".to_string()
				}
			}


			if cfg!( debug_assertions ) { println!("break from Xi: {:?}-{:?}", start, end ) }
			let mut s = SplitPoint::new( start, end, self.priority );
			s.mandatory = hard;
			s.glue      = glue;

			s
		})

		.collect::< Vec<_> >()


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
	}
}



#[cfg(test)]
mod tests
{
	use super::*;


	fn end( start: usize, stop: usize, prio: usize ) -> SplitPoint
	{
		let mut result   = SplitPoint::new( start, stop, prio );
		result.glue      = "".to_string();
		result.mandatory = true;

		result
	}

	#[test]
	fn basic() {

		assert_eq!( Xi{ priority: 0 }.opportunities( "foo bar" ), vec![ SplitPoint::new( 3, 4, 0 ), end( 7, 7, 0 ) ] ); }


	#[test]
	fn more_spaces() {

		assert_eq!( Xi{ priority: 0 }.opportunities( "foo   bar" ), vec![ SplitPoint::new( 3, 6, 0 ), end( 9, 9, 0 ) ] ); }


	#[test]
	fn no_break_space() {

		assert_eq!( Xi{ priority: 0 }.opportunities( "foo\u{A0}bar" ), vec![ end( 8, 8, 0 ) ] ); }


	#[test]
	fn tabstop() {

		assert_eq!( Xi{ priority: 0 }.opportunities( "foo\tbar" ), vec![ SplitPoint::new( 3, 4, 0 ), end( 7, 7, 0 ) ] ); }


	#[test]
	fn hyphens()
	{
		// \u{ad} is Unicode U+00AD SOFT HYPHEN
		//
		assert_eq!
		(
			  Xi{ priority: 0 }.opportunities( "co\u{ad}ca-co‧la" )

			, vec!
			  [
				  SplitPoint::new(  4,  4, 0 ) ,
				  SplitPoint::new(  7,  7, 0 ) ,
				  SplitPoint::new( 12, 12, 0 ) ,
				  end            ( 14, 14, 0 ) ,
			  ]
		);
	}


	// I don't know if this is really desirable behaviour, but in the worst case splits like this can be prevented with a filter.
	//
	#[test]
	fn hyphen_series()
	{
		assert_eq!( Xi{ priority: 0 }.opportunities( "bin--doo" ), vec![ SplitPoint::new( 5, 5, 0 ), end( 8, 8, 0 ) ] );
	}


	#[test]
	fn newline()
	{
		let mut split   = SplitPoint::new( 3, 4, 0 );
		split.mandatory = true;

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\ndoo" ), vec![ split, end( 7, 7, 0 ) ] );
	}


	#[test]
	fn vertical_tab()
	{
		let mut split   = SplitPoint::new( 3, 4, 0 );

		split.mandatory = true;
		split.glue      = "\u{000B}".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\u{000B}doo" ), vec![ split, end( 7, 7, 0 ) ] );
	}


	#[test]
	fn form_feed()
	{
		let mut split   = SplitPoint::new( 3, 4, 0 );

		split.mandatory = true;
		split.glue      = "\u{000C}".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\u{000C}doo" ), vec![ split, end( 7, 7, 0 ) ] );
	}


	#[test]
	fn cariage_return()
	{
		let mut split   = SplitPoint::new( 3, 4, 0 );

		split.mandatory = true;
		split.glue      = "\u{000D}".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\u{000D}doo" ), vec![ split, end( 7, 7, 0 ) ] );
	}


	#[test]
	fn next_line()
	{
		let mut split   = SplitPoint::new( 3, 5, 0 );

		split.mandatory = true;
		split.glue      = "\u{0085}".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\u{0085}doo" ), vec![ split, end( 8, 8, 0 ) ] );
	}


	#[test]
	fn line_separator()
	{
		let mut split   = SplitPoint::new( 3, 6, 0 );

		split.mandatory = true;
		split.glue      = "\u{2028}".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\u{2028}doo" ), vec![ split, end( 9, 9, 0 ) ] );
	}


	#[test]
	fn paragraph_separator()
	{
		let mut split   = SplitPoint::new( 3, 6, 0 );

		split.mandatory = true;
		split.glue      = "\u{2029}".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\u{2029}doo" ), vec![ split, end( 9, 9, 0 ) ] );
	}


	#[test]
	fn consecutive_newlines_should_produce_more_than_one_splitpoint()
	{
		let mut split  = SplitPoint::new( 3, 4, 0 );
		let mut split2 = SplitPoint::new( 4, 5, 0 );

		split .mandatory = true;
		split2.mandatory = true;

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\n\ndoo" ), vec![ split, split2, end( 8, 8, 0 ) ] );
	}


	#[test]
	fn crlf_should_give_one_splitpoint()
	{
		let mut split  = SplitPoint::new( 3, 5, 0 );

		split.mandatory = true;
		split.glue      = "\r\n".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\r\ndoo" ), vec![ split, end( 8, 8, 0 ) ] );
	}


	// For now the newline is marked, but not consumed
	//
	#[test]
	fn lfcrlf_shouldnt_eat_first_lf()
	{
		let mut split  = SplitPoint::new( 3, 4, 0 );
		let mut split2 = SplitPoint::new( 4, 6, 0 );

		split .mandatory = true;
		split2.mandatory = true;

		split .glue      =   "\n".to_string();
		split2.glue      = "\r\n".to_string();

		assert_eq!( Xi{ priority: 0 }.opportunities( "bin\n\r\ndoo" ), vec![ split, split2, end( 9, 9, 0 ) ] );
	}


	// #[test]
	// fn tabstop2()
	// {
	// 	assert_eq!
	// 	(
	// 		  Xi{ priority: 0 }.opportunities( "foo \t bar" )

	// 		  , vec!
	// 		    [
	// 		  	    SplitPoint { start: ByteOffset( 3 ), end: ByteOffset( 6 ), glue: "", mandatory: false, priority: 0, width: None }
	// 		    ]
	// 	);
	// }
}
