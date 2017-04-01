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
	pub fn wrap_line( &self, line: &str ) -> Vec< String >
	{
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


		// Choose which split points we will actually use
		//
		// The offset where the current line starts in display width
		//
		let mut width_offset = WidthOffset( 0 );

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


			let mut found: Option< &SplitPoint > = None ;
			let mut last_score                   = 0    ;

			// Figure out the last valid split point for each priority for this line.
			//
			for split in &splits
			{
				println!( "Considering: start: {:?}, end: {:?}", split.start, split.end );

				// Byte to width conversions will round down, so we shouldn't use <= here. The last splitpoint, at the end of the string
				// which we should never use, shall point to the width of the last character, since that is the last valid width.
				//
				if split.width.unwrap() < endl
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
			//
			assert_ne!( cut.start.0, line.len() );

			let mut s = line[ start..cut.start.0 ].to_string();

			s.push_str( cut.glue );

			out.push( s );
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


	fn standard( string: &str, width: usize ) -> Vec< String >
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


	// #[test]
	// fn basic()
	// {
	// 	assert_eq!( standard( "ha ha ah", 3 ), vec![ "ha", "ha", "ah" ] );
	// }


	// #[test]
	// fn consecutive_spaces()
	// {
	// 	assert_eq!( standard( "ha ha       ah", 3 ), vec![ "ha", "ha", "ah" ] );
	// }


	#[test]
	fn nbsp()
	{
		println!("{:?}","foo b\u{A0}r baz" );
		assert_eq!( standard( "foo b\u{A0}r baz", 6 ), vec![ "foo", "b\u{A0}r", "baz" ] );
	}
}
