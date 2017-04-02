use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use super::*;

use generator::Generate  ;
use filter   ::Filter    ;
use ruler    ::TextWidth ;


pub struct Wrapper<'a, 'b, Ruler>
{
	pub width     : usize               ,
	pub generators: Vec< &'a Generate > ,
	pub filters   : Vec< &'b Filter   > ,
	pub ruler     : Ruler               ,
}


impl<'a, 'b, Ruler> Wrapper<'a, 'b, Ruler>

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

			if cfg!( debug_assertions ) { println!( "byte: {:02?}, width: {:02?}, graph:{:?}", bytes, width.0, graph ) }

			width = width + self.ruler.measure( graph );
		}

		// Add the end of the string
		//
		b2w.insert( ByteOffset( line.len() ), width                     );
		w2b.insert( width                   , ByteOffset( line.len()  ) );

		if cfg!( debug_assertions ) { println!( "byte: {:02?}, width: {:02?}\n", line.len(), width.0 ) }


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
				split.width = Some( split.start.to_width( &b2w ) + self.ruler.measure( &split.glue ) );

				splits.push( split );
			}
		}


		// Let filters do their work on the splits
		//
		for filter in &self.filters
		{
			filter.run( line, &mut splits );
		}


		// Sort the split points
		// SplitPoints will be sorted on a score calculated by adding the start offset in bytes to the priority.
		// This means that for a certain set of splitpoints, for which width + glue.width are within desired width, the
		// splitpoint with the highest score will appear last in the vector.
		//
		splits.sort();

		if cfg!( debug_assertions )
		{
			println!("Available splits:");

			for split in & splits
			{
				println!( "start: {:?}, end: {:?}", split.start.0, split.end.0 );
			}

			println!("");
		}


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

			if cfg!( debug_assertions ) { println!("width_offset: {:?}, endl: {:?}, line_width: {:?}", width_offset.0, endl.0, line_width.0 ) }


			let mut found: Option< &SplitPoint > = None ;
			let mut last_score                   = 0    ;

			// Figure out the last valid split point for each priority for this line.
			//
			for (i, split) in splits[ candidate.. ].iter().enumerate()
			{
				if cfg!( debug_assertions ) { println!( "Considering: start: {:?}, end: {:?} with endl: {:?}, score: {:?}", split.start.0, split.end.0, endl.0, split.score() ) }


				if split.width.unwrap() <= endl
				{
					if !split.enabled { continue }

					else if split.mandatory
					{
						found = Some( split );
						candidate += i + 1   ;
						break;
					}

					else if split.score() >= last_score
					{
						found      = Some( split ) ;
						last_score = split.score() ;
					}

					else { continue }
				}

				else
				{
					candidate += i;
					break
				}
			}


			if found.is_some()
			{
				let split = found.unwrap();

				if cfg!( debug_assertions ) { println!("Found: {:?}, {:?}", split.start, split.end ); }

				width_offset = split.end.to_width( &b2w );
				cuts.push( split )
			}


			else
			{
				panic!("No valid split point found" );
			}

		}


		if cfg!( debug_assertions )
		{
			for c in &cuts
			{
				println!( "{:?}", c );
			}
			println!("");
		}


		let mut out       = Vec::with_capacity( cuts.len() + 1 );
		let mut start     = 0;

		for cut in cuts
		{
			// We should never try to cut at the end of the string, but it happens.
			// After some time, this can be commented out.
			//
			debug_assert!( cut.start.0 != line.len() );

			let mut s = line[ start..cut.start.0 ].to_string();

			s.push_str( &cut.glue );

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


	pub fn wrap( &self, input: &str ) -> Vec< String >
	{
		let mut out = Vec::new();

		for line in input.lines()
		{
			out.append( &mut self.wrap_line( line ) );
		}

		out
	}


	pub fn fill( &self, input: &str ) -> String
	{
		self.wrap( input ).join( "\n" )
	}
}




#[cfg(test)]
mod tests
{
	use super::*;

	use generator::unicode_standard::Xi           ;
	use generator::hyphenation     ::Hyphenator   ;
	use hyphenation_crate          ::Language     ;
	use ruler::unicode_width       ::UnicodeWidth ;


	//--------------------
	// basic xi splitting
	//
	fn xi( string: &str, width: usize, prio: usize ) -> Vec< String >
	{
		let gen     = Xi{ priority: prio };

		let wrapper = Wrapper
		{
			width     : width        ,
			generators: vec![ &gen ] ,
			ruler     : UnicodeWidth ,
			filters   : Vec::new()   ,
		};

		wrapper.wrap_line( string )
	}



	#[test]
	fn basic()
	{
		assert_eq!( xi( "ha ha ah", 3, 1 ), vec![ "ha", "ha", "ah" ] );
	}


	#[test]
	fn basic_zero_priority()
	{
		assert_eq!( xi( "ha ha ah", 3, 0 ), vec![ "ha", "ha", "ah" ] );
	}


	#[test]
	fn basic_high_priority()
	{
		assert_eq!( xi( "ha ha ah", 3, 99999 ), vec![ "ha", "ha", "ah" ] );
	}



	#[test]
	fn consecutive_spaces()
	{
		assert_eq!( xi( "ha ha       ah", 3, 1 ), vec![ "ha", "ha", "ah" ] );
	}



	#[test]
	fn consecutive_spaces_and_tabs()
	{
		assert_eq!( xi( "ha ha   \t   ah", 3, 1 ), vec![ "ha", "ha", "ah" ] );
	}



	#[test]
	fn nbsp()
	{
		assert_eq!( xi( "foo b\u{A0}r baz", 6, 1 ), vec![ "foo", "b\u{A0}r", "baz" ] );
	}


	#[test]
	fn dont_split_every_space()
	{
		assert_eq!( xi( "foo bar baz fiend", 9, 1 ), vec![ "foo bar", "baz fiend" ] );
	}


	#[test]
	fn width_zero()
	{
		#![should_panic]
		xi( "foo bar baz", 0, 1 );
	}


	#[test]
	fn whitespace_should_not_be_squeezed()
	{
		assert_eq!( xi( "foo \t a bar", 7, 1 ), vec![ "foo \t a", "bar" ] );
	}


	#[test]
	fn whitespace_should_be_trimmed()
	{
		assert_eq!( xi( "foo \t  bar  ", 10, 1 ), vec![ "foo \t  bar" ] );
	}


	#[test]
	fn whitespace_should_not_be_trimmed_left_on_first_line()
	{
		assert_eq!( xi( " \tfoo \t  bar  ", 4, 1 ), vec![ " \tfoo", "bar" ] );
	}


	#[test]
	fn leadingspaces_blocking_split()
	{
		assert_eq!( xi( " a b c", 1, 1 ), vec![ "a", "b", "c" ] );
	}


	#[test]
	fn whitespace_should_be_trimmed_on_every_line_yet_no_empty_strings_should_exist_in_output()
	{
		assert_eq!( xi( "foo   ssss bars", 4, 1 ), vec![ "foo", "ssss", "bars" ] );
	}

	#[test] #[should_panic]
	fn dont_break_before_punctuation()
	{
		xi( "a ! b : c ? d", 2, 0 );
	}


	#[test]
	fn hyphens()
	{
		assert_eq!( xi( "co\u{ad}ca-co‧la", 3, 1 ), vec![ "co\u{ad}", "ca-", "co‧", "la" ] );
	}

	//-------------
	// Hyphenation
	//
	fn hyphenate( string: &str, width: usize ) -> Vec< String >
	{
		let c   = hyphenation_crate::load( Language::English_US ).unwrap();
		let gen = Hyphenator{ priority: 1, corpus: &c, glue: "-".to_string() };

		let wrapper = Wrapper
		{
			width     : width        ,
			generators: vec![ &gen ] ,
			ruler     : UnicodeWidth ,
			filters   : Vec::new()   ,
		};

		wrapper.wrap_line( string )
	}


	#[test]
	fn hyphenation()
	{
		assert_eq!( hyphenate( "hyphenation", 7 ), vec![ "hyphen-", "ation"          ] );
		assert_eq!( hyphenate( "hyphenation", 6 ), vec![ "hy-"    , "phen-", "ation" ] );
		assert_eq!( hyphenate( "hyphenation", 5 ), vec![ "hy-"    , "phen-", "ation" ] );
	}


	#[test] #[should_panic]
	fn too_short()
	{
		hyphenate( "hyphenation", 4 );
	}


	//----------------------
	// Combining Generators
	//
	fn combine( string: &str, width: usize, hyph_prio: usize, xi_prio: usize ) -> Vec< String >
	{
		let c    = hyphenation_crate::load( Language::English_US ).unwrap();
		let hyph = Hyphenator{ priority: hyph_prio, corpus: &c, glue: "-".to_string() };

		let xi   = Xi{ priority: xi_prio };

		let wrapper = Wrapper
		{
			width     : width              ,
			generators: vec![ &hyph, &xi ] ,
			ruler     : UnicodeWidth       ,
			filters   : Vec::new()         ,
		};

		let normal = wrapper.wrap_line( string );

		let reverse = Wrapper
		{
			width     : width              ,
			generators: vec![ &xi, &hyph ] ,
			ruler     : UnicodeWidth       ,
			filters   : Vec::new()         ,
		};

		let reversed = reverse.wrap_line( string );

		assert_eq!( normal, reversed );

		normal
	}

	#[test]
	fn combine_generators_basic()
	{
		assert_eq!( combine( "hyphenation is key", 7, 0, 0 ), vec![ "hyphen-", "ation", "is key" ] );
	}

	#[test]
	fn combine_priority()
	{
		assert_eq!( combine( "the hyphenation is key", 7, 0, 0 ), vec![ "the hy-", "phen-"  , "ation", "is key" ] );
		assert_eq!( combine( "the hyphenation is key", 7, 0, 3 ), vec![ "the hy-", "phen-"  , "ation", "is key" ] );
		assert_eq!( combine( "the hyphenation is key", 7, 0, 4 ), vec![ "the"    , "hyphen-", "ation", "is key" ] );
	}


	//---------------------------------
	// Combining Generators and filters
	//
	fn combine_filter( string: &str, width: usize, hyph_prio: usize, xi_prio: usize ) -> Vec< String >
	{
		let c       = hyphenation_crate::load( Language::English_US ).unwrap();
		let hyph    = Hyphenator{ priority: hyph_prio, corpus: &c, glue: "-".to_string() };

		let xi      = Xi{ priority: xi_prio };

		let french  = filter::french::French;

		let wrapper = Wrapper
		{
			width     : width              ,
			generators: vec![ &hyph, &xi ] ,
			ruler     : UnicodeWidth       ,
			filters   : vec![ &french ]    ,
		};

		let normal = wrapper.wrap_line( string );

		let reverse = Wrapper
		{
			width     : width              ,
			generators: vec![ &xi, &hyph ] ,
			ruler     : UnicodeWidth       ,
			filters   : vec![ &french ]    ,
		};

		let reversed = reverse.wrap_line( string );

		assert_eq!( normal, reversed );

		normal
	}

	#[test]
	fn married_with_filters()
	{
		assert_eq!( combine       ( "hyphenation « is k »", 7, 0, 0 ), vec![ "hyphen-", "ation «", "is k »"    ] );
		assert_eq!( combine_filter( "hyphenation « is k »", 7, 0, 0 ), vec![ "hyphen-", "ation", "« is", "k »" ] );
	}


	// fn lorem_ipsum(length: usize) -> &'static str {
	//     let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas feugiat non mi \
	//                 rutrum consectetur. Nulla iaculis luctus ex suscipit posuere. Sed et tellus quis \
	//                 elit volutpat pretium. Sed faucibus purus vitae feugiat tincidunt. Nulla \
	//                 malesuada interdum tempus. Proin consectetur malesuada magna, id suscipit enim \
	//                 tempus in. Sed sollicitudin velit tortor, quis condimentum nisl vulputate \
	//                 lobortis. Curabitur id lectus arcu. Nullam quis aliquam nisi. Vestibulum quam \
	//                 enim, elementum vel urna scelerisque, ultricies cursus urna. Mauris vestibulum, \
	//                 augue non posuere viverra, risus tortor iaculis augue, eget convallis metus nisl \
	//                 vestibulum nisi. Aenean auctor dui vel aliquet sagittis. Aliquam quis enim \
	//                 mauris. Nunc eu leo et orci euismod bibendum vel eu tortor. Nam egestas volutpat \
	//                 ex, a turpis duis.";
	//     text.split_at(length).0
	// }


	// #[test]
	// fn bench_test()
	// {
	//     let c    = hyphenation_crate::load( Language::Latin ).unwrap();
	//     let hyph = Hyphenator{ priority: 0, corpus: &c, glue: "-".to_string() };
	//     let xi   = Xi{ priority: 0 };
	//     let w    = Wrapper
	//     {
	//         width     : 60                 ,
	//         generators: vec![ &hyph, &xi ] ,
	//         ruler     : UnicodeWidth       ,
	//         filters   : vec![]             ,
	//     };

	//     let text = lorem_ipsum( 800 );

	//     assert_eq!( w.fill( text ), String::new() );
	// }

}
