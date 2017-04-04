use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use super::*;


pub struct Wrapper<Ruler>
{
	width     : usize                ,
	generators: Vec< Box<Generate> > ,
	filters   : Vec< Box<Filter>   > ,
	ruler     : Ruler                ,
}


impl<Ruler> Wrapper<Ruler> where Ruler: TextWidth
{
	pub fn new( width: usize, generators: Vec< Box<Generate> >, filters: Vec< Box<Filter> >, ruler: Ruler )

	-> Result< Wrapper<Ruler>, &'static str >
	{
		if width == 0 { return Err( "Wrapper.width cannot be zero" ) }

		Ok
		(
			Wrapper
			{
				width     : width     ,
				generators: generators,
				filters   : filters   ,
				ruler     : ruler     ,
			}
		)
	}


	pub fn width( &self ) -> usize { self.width }


	pub fn set_width( &mut self, width: usize )

	-> Result< (), &'static str >
	{
		if width == 0 { return Err( "Wrapper.width cannot be zero" ) }

		self.width = width;

		Ok(())
	}


	pub fn wrap( &self, line: &str ) -> Result< String, &'static str >
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

			if cfg!( debug_assertions ) { println!( "byte: {:02?}, width: {:02?}, graph:{:?}", bytes, width.0, graph ) }

			width = width + self.ruler.measure( graph );
		}

		// Add the end of the string
		//
		b2w.insert( ByteOffset( line.len() ), width                     );
		w2b.insert( width                   , ByteOffset( line.len()  ) );

		if cfg!( debug_assertions ) { println!( "byte: {:02?}, width: {:02?}\n", line.len(), width.0 ) }


		let line_width = width ;

		if line_width.0 <= self.width { return Ok( line.to_string() ) }

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
			println!( "Available splits:" );

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


			let mut found: Option< &SplitPoint > = None             ;
			let mut last_score                   = WidthOffset( 0 ) ;

			// Figure out the last valid split point for each priority for this line.
			//
			for (i, split) in splits[ candidate.. ].iter().enumerate()
			{
				if cfg!( debug_assertions ) { println!( "Considering: start: {:?}, end: {:?} with endl: {:?}, score: {:?}", split.start.0, split.end.0, endl.0, split.score( &self.ruler ) ) }


				if split.width.unwrap() <= endl
				{
					if !split.enabled { continue }

					else if split.mandatory
					{
						found = Some( split );
						candidate += i + 1   ;
						break;
					}

					else if split.score( &self.ruler ) >= last_score
					{
						found      = Some( split ) ;
						last_score = split.score( &self.ruler ) ;
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

				if cfg!( debug_assertions ) { println!( "Found: {:?}, {:?}", split.start, split.end ); }

				width_offset = split.end.to_width( &b2w );
				cuts.push( split )
			}


			else{ return Err( "No valid split point found" ); }
		}


		if cfg!( debug_assertions )
		{
			for c in &cuts
			{
				println!( "{:?}", c );
			}
			println!("");
		}


		let mut out       = String::with_capacity( line.len() + cuts.len() * 2 );
		let mut start     = 0;

		for cut in cuts
		{
			// We should never try to cut at the end of the string, but it happens.
			// After some time, this can be commented out.
			//
			debug_assert!( cut.start.0 != line.len() );

			out.push_str( &line[ start..cut.start.0 ] );


			if cut.start.0 != 0  &&  cut.end.0 != line.len() {

				out.push_str( &cut.glue ); }


			start = cut.end.0;
		}

		out.push_str( &line[ start..line.len() ] );

		Ok( out )

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
	fn xi( string: &str, width: usize, prio: usize ) -> Result< String, &'static str >
	{
		let gen     = Box::new( Xi{ priority: prio } );

		let wrapper = try!( Wrapper::new( width, vec![ gen ], Vec::new(), UnicodeWidth ) );

		wrapper.wrap( string )
	}



	#[test]
	fn width()
	{
		let w = Wrapper::new( 5, Vec::new(), Vec::new(), UnicodeWidth ).unwrap();
		assert_eq!( w.width(), 5 );
	}



	#[test]
	fn basic()
	{
		assert_eq!( xi( "ha ha ah", 3, 1 ), Ok( "ha\nha\nah".to_string() ) );
	}


	#[test]
	fn basic_zero_priority()
	{
		assert_eq!( xi( "ha ha ah", 3, 0 ), Ok( "ha\nha\nah".to_string() ) );
	}


	#[test]
	fn basic_high_priority()
	{
		assert_eq!( xi( "ha ha ah", 3, 99999 ), Ok( "ha\nha\nah".to_string() ) );
	}



	#[test]
	fn consecutive_spaces()
	{
		assert_eq!( xi( "ha ha       ah", 3, 1 ), Ok( "ha\nha\nah".to_string() ) );
	}



	#[test]
	fn consecutive_spaces_and_tabs()
	{
		assert_eq!( xi( "ha ha   \t   ah", 3, 1 ), Ok( "ha\nha\nah".to_string() ) );
	}



	#[test]
	fn nbsp()
	{
		assert_eq!( xi( "foo b\u{A0}r baz", 6, 1 ), Ok( "foo\nb\u{A0}r\nbaz".to_string() ) );
	}


	#[test]
	fn dont_split_every_space()
	{
		assert_eq!( xi( "foo bar baz fiend", 9, 1 ), Ok( "foo bar\nbaz fiend".to_string() ) );
	}


	#[test]
	fn width_zero()
	{
		assert_eq!( xi( "foo bar baz", 0, 1 ), Err( "Wrapper.width cannot be zero" ) );
	}


	#[test]
	fn whitespace_should_not_be_squeezed()
	{
		assert_eq!( xi( "foo \t a bar", 7, 1 ), Ok( "foo \t a\nbar".to_string() ) );
	}


	#[test]
	fn whitespace_should_not_be_trimmed_left_on_first_line()
	{
		assert_eq!( xi( " \tfoo \t  bar  ", 4, 1 ), Ok( " \tfoo\nbar\n  ".to_string() ) );
	}


	#[test]
	fn leadingspaces_blocking_split()
	{
		assert_eq!( xi( " a b c", 1, 1 ), Ok( "a\nb\nc".to_string() ) );
	}


	#[test]
	fn whitespace_should_be_trimmed_on_every_line_yet_no_empty_strings_should_exist_in_output()
	{
		assert_eq!( xi( "foo   ssss bars", 4, 1 ), Ok( "foo\nssss\nbars".to_string() ) );
	}


	#[test]
	fn dont_break_before_punctuation()
	{
		assert_eq!( xi( "a ! b : c ? d", 2, 0 ), Err( "No valid split point found" ) );
	}


	#[test]
	fn hyphens()
	{
		assert_eq!( xi( "co\u{ad}ca-co‧la", 3, 1 ), Ok( "co\u{ad}\nca-\nco‧\nla".to_string() ) );
	}


	#[test]
	fn newlines_should_not_be_dropped()
	{
		assert_eq!( xi( "co\n\n\nla", 3, 1 ), Ok( "co\n\n\nla".to_string() ) );
	}


	#[test]
	fn leading_and_trailing_newlines_should_not_be_dropped()
	{
		assert_eq!( xi( "\ncola\n", 4, 1 ), Ok( "\ncola\n".to_string() ) );
	}

	//-------------
	// Hyphenation
	//
	fn hyphenate( string: &str, width: usize ) -> Result< String, &'static str >
	{
		let c   = hyphenation_crate::load( Language::English_US ).unwrap();
		let gen = Box::new( Hyphenator{ priority: 1, corpus: c, glue: "-\n".to_string() } );

		let wrapper = try!( Wrapper::new( width, vec![ gen ], Vec::new(), UnicodeWidth ) );

		wrapper.wrap( string )
	}


	#[test]
	fn hyphenation()
	{
		assert_eq!( hyphenate( "hyphenation", 7 ), Ok( "hyphen-\nation"   .to_string() ) );
		assert_eq!( hyphenate( "hyphenation", 6 ), Ok( "hy-\nphen-\nation".to_string() ) );
		assert_eq!( hyphenate( "hyphenation", 5 ), Ok( "hy-\nphen-\nation".to_string() ) );
	}


	#[test]
	fn too_short()
	{
		assert_eq!( hyphenate( "hyphenation", 4 ), Err( "No valid split point found" ) );
	}


	//----------------------
	// Combining Generators
	//
	fn combine( string: &str, width: usize, hyph_prio: usize, xi_prio: usize ) -> Result< String, &'static str >
	{
		let c    = hyphenation_crate::load( Language::English_US ).unwrap();
		let hyph = Box::new( Hyphenator{ priority: hyph_prio, corpus: c, glue: "-\n".to_string() } );
		let xi   = Box::new( Xi{ priority: xi_prio } );

		let reverse = try!( Wrapper::new( width, vec![ xi.clone(), hyph.clone() ], Vec::new(), UnicodeWidth ) );
		let wrapper = try!( Wrapper::new( width, vec![ hyph      , xi           ], Vec::new(), UnicodeWidth ) );

		let normal   = wrapper.wrap( string );
		let reversed = reverse.wrap( string );

		assert_eq!( normal, reversed );

		normal
	}


	#[test]
	fn combine_generators_basic()
	{
		assert_eq!( combine( "hyphenation is key", 7, 0, 0 ), Ok( "hyphen-\nation\nis key".to_string() ) );
	}

	#[test]
	fn combine_priority()
	{
		assert_eq!( combine( "the hyphenation is key", 7, 0, 0 ), Ok( "the hy-\nphen-\nation\nis key".to_string() ) );
		assert_eq!( combine( "the hyphenation is key", 7, 0, 3 ), Ok( "the hy-\nphen-\nation\nis key".to_string() ) );
		assert_eq!( combine( "the hyphenation is key", 7, 0, 4 ), Ok( "the\nhyphen-\nation\nis key".to_string() ) );
	}

// 	#[test]
// 	fn multiline_to_wrapline()
// 	{
// 		let example = "Memory
// safety";

// 		println!("{:?}", example );

// 		// assert!( false );

// 		assert_eq!( combine( example, 15, 0, 0 ), "" );
// 	}


	//---------------------------------
	// Combining Generators and filters
	//
	fn combine_filter( string: &str, width: usize, hyph_prio: usize, xi_prio: usize ) -> Result< String, &'static str >
	{
		let c       = hyphenation_crate::load( Language::English_US ).unwrap();
		let hyph    = Box::new( Hyphenator{ priority: hyph_prio, corpus: c, glue: "-\n".to_string() } );

		let xi      = Box::new( Xi{ priority: xi_prio } );

		let french  = Box::new( filter::french::French );

		let wrapper = Wrapper
		{
			width     : width                            ,
			generators: vec![ hyph.clone(), xi.clone() ] ,
			ruler     : UnicodeWidth                     ,
			filters   : vec![ french.clone() ]           ,
		};

		let normal = wrapper.wrap( string );

		let reverse = Wrapper
		{
			width     : width              ,
			generators: vec![ xi, hyph ] ,
			ruler     : UnicodeWidth       ,
			filters   : vec![ french ]    ,
		};

		let reversed = reverse.wrap( string );

		assert_eq!( normal, reversed );

		normal
	}

	#[test]
	fn married_with_filters()
	{
		assert_eq!( combine       ( "hyphenation « is k »", 7, 0, 0 ), Ok( "hyphen-\nation «\nis k »" .to_string() ) );
		assert_eq!( combine_filter( "hyphenation « is k »", 7, 0, 0 ), Ok( "hyphen-\nation\n« is\nk »".to_string() ) );
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
