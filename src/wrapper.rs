use std::collections::HashMap;
use std::io::Write;
use std::io;

use unicode_segmentation::UnicodeSegmentation;

use super::*;

use generator::interface::Generator;
use ruler    ::interface::TextWidth;


pub struct Wrapper<'a, Ruler>
{
	pub width     : usize                ,
	pub generators: Vec< &'a Generator > ,
	pub ruler     : Ruler                ,
}


impl<'a, Ruler> Wrapper<'a, Ruler>

	where Ruler: TextWidth
{
	pub fn wrap_line( &self, input: &str ) -> Vec< String >
	{
		let line = input.trim_right();

		// store byte to width conversion, because we will need to calculate our breakpoint in terms of display width.
		//
		let mut b2w: HashMap < ByteOffset , WidthOffset > = HashMap::with_capacity( line.len() );
		let mut w2b: HashMap < WidthOffset, ByteOffset  > = HashMap::with_capacity( line.len() );

		let mut width = WidthOffset( 0 );

		for ( bytes, graph ) in line.grapheme_indices( true )
		{
			b2w.insert( ByteOffset( bytes ), width                );
			w2b.insert( width              , ByteOffset( bytes  ) );

			println!( "byte: {:02?}, width: {:02?}, graph:{:?}", bytes, width.0, graph );
			io::stdout().flush().ok().expect("Could not flush stdout");
			width = width + self.ruler.measure( graph );
		}

		// Add the end of the string
		//
		b2w.insert( ByteOffset( line.len() ), width                     );
		w2b.insert( width                   , ByteOffset( line.len()  ) );

		println!( "byte: {:02?}, width: {:02?}\n", line.len(), width.0 );


		let line_width = width ;

		if line_width.0 <= self.width { return vec![ line.to_string() ] }

		let mut splits: Vec< SplitPoint > = Vec::with_capacity( line_width.0 );


		// Harvest the split points from the generators
		//
		for generator in &self.generators
		{
			// TODO: shouldn't generator return an iterator rather than a vector?
			//
			for mut split in generator.opportunities( line )
			{
				split.width = Some( split.start.to_width( &b2w ) + self.ruler.measure( split.glue ) );

				splits.push( split );
			}
		}

		// Sort the split points
		// SplitPoints will be sorted on a score calculated by adding the start offset in bytes to the priority.
		// This means that for a certain set of splitpoints, for which width + glue.width are within desired width, the
		// splitpoint with the highest score will appear last in the vector.
		//
		splits.sort();

		println!("Available splits:");

		for split in & splits
		{
			println!( "start: {:?}, end: {:?}", split.start.0, split.end.0 );
		}

		println!("");


		// Choose which split points we will actually use
		//
		// The offset where the current line starts in display width
		//
		let mut width_offset = WidthOffset( 0 );

		// This is the index of the first split after we found one. This avoids re-considering splits several times.
		//
		let mut candidate    = 0;

		// The actual split points that will be used to produce the return value.
		// We probably won't be able to cut at ideal widths, so we might need an extra line, so plus one.
		//
		let mut cuts: Vec< &SplitPoint > = Vec::with_capacity( line_width.0 / self.width + 1 );


		loop
		{
			// Do not search for a split point if the rest of the string fits in the current line.
			//
			let endl = width_offset + self.width;

			// If what we have left fits in one line, we are done.
			//
			if endl >= line_width { break }

			println!("width_offset: {:?}, endl: {:?}, line_width: {:?}", width_offset.0, endl.0, line_width.0 );


			let mut found: Option< &SplitPoint > = None ;
			let mut last_score                   = 0    ;

			// Figure out the last valid split point for each priority for this line.
			//
			for (i, split) in splits[ candidate.. ].iter().enumerate()
			{
				println!( "Considering: start: {:?}, end: {:?} with endl: {:?}", split.start.0, split.end.0, endl.0 );

				// Byte to width conversions will round down, so we shouldn't use <= here. The last splitpoint, at the end of the string
				// which we should never use, shall point to the width of the last character, since that is the last valid width.
				//
				if split.width.unwrap() <= endl
				{
					if split.mandatory
					{
						found = Some( split );
						break;
					}

					else if split.score() >= last_score
					{
						found      = Some( split ) ;
						last_score = split.score() ;
						candidate  = i + 1         ;
					}

					else { continue }
				}

				else { break }
			}


			if found.is_some()
			{
				let split = found.unwrap();

				println!("Found: {:?}, {:?}", split.start, split.end );

				width_offset = split.end.to_width( &b2w );
				cuts.push( split )
			}


			else
			{
				panic!("No valid split point found" );
			}

		}


		for c in &cuts
		{
			println!( "{:?}", c );
		}
		println!("");

		let mut out       = Vec::with_capacity( cuts.len() + 1 );
		let mut start     = 0;

		for cut in cuts
		{
			// We should never try to cut at the end of the string, but it happens.
			// After some time, this can be commented out.
			//
			assert_ne!( cut.start.0, line.len() );

			let mut s = line[ start..cut.start.0 ].to_string();

			s.push_str( cut.glue );

			// We should never store empty strings, it might happen, check test: leadingspaces_blocking_split
			//
			if !s.is_empty() { out.push( s ) }

			// This needs to happen even if the string was empty, because it might have eaten white space.
			//
			start = cut.end.0;
		}

		out.push( line[ start..line.len() ].to_string() );

		out

	}
}




#[cfg(test)]
mod tests
{
	use super::*;

	use generator::unicode_standard::Xi;
	use generator::interface::Generator;


	fn xi( string: &str, width: usize ) -> Vec< String >
	{
		let gen  = Xi{ priority: 1 };
		let gens = vec![ &gen as &Generator ];

		let wrapper = Wrapper
		{
			width     : width                             ,
			generators: gens                              ,
			ruler     : ruler::unicode_width::UnicodeWidth,
		};

		wrapper.wrap_line( string )
	}



	#[test]
	fn basic()
	{
		assert_eq!( xi( "ha ha ah", 3 ), vec![ "ha", "ha", "ah" ] );
	}



	#[test]
	fn consecutive_spaces()
	{
		assert_eq!( xi( "ha ha       ah", 3 ), vec![ "ha", "ha", "ah" ] );
	}



	#[test]
	fn consecutive_spaces_and_tabs()
	{
		assert_eq!( xi( "ha ha   \t   ah", 3 ), vec![ "ha", "ha", "ah" ] );
	}



	#[test]
	fn nbsp()
	{
		println!("{:?}","foo b\u{A0}r baz" );
		assert_eq!( xi( "foo b\u{A0}r baz", 6 ), vec![ "foo", "b\u{A0}r", "baz" ] );
	}


	#[test]
	fn dont_split_every_space()
	{
		assert_eq!( xi( "foo bar baz fiend", 9 ), vec![ "foo bar", "baz fiend" ] );
	}


	#[test]
	fn width_zero()
	{
		#![should_panic]
		xi( "foo bar baz", 0 );
	}


	#[test]
	fn whitespace_should_not_be_squeezed()
	{
		assert_eq!( xi( "foo \t a bar", 7 ), vec![ "foo \t a", "bar" ] );
	}


	#[test]
	fn whitespace_should_be_trimmed()
	{
		assert_eq!( xi( "foo \t  bar  ", 10 ), vec![ "foo \t  bar" ] );
	}


	#[test]
	fn whitespace_should_not_be_trimmed_left_on_first_line()
	{
		assert_eq!( xi( " \tfoo \t  bar  ", 4 ), vec![ " \tfoo", "bar" ] );
	}


	#[test]
	fn leadingspaces_blocking_split()
	{
		assert_eq!( xi( " a b c", 1 ), vec![ "a", "b", "c" ] );
	}


	#[test]
	fn whitespace_should_be_trimmed_on_every_line_yet_no_empty_strings_should_exist_in_output()
	{
		assert_eq!( xi( "foo   ssss bars", 4 ), vec![ "foo", "ssss", "bars" ] );
	}
}
