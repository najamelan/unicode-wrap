

extern crate hyphenation as hyphenation;
extern crate unicode_wrap;

use hyphenation::Language;

use unicode_wrap::Wrapper;

use unicode_wrap :: generator :: unicode_standard :: Xi           ;
use unicode_wrap :: generator :: hyphenation      :: Hyphenator   ;
use unicode_wrap :: ruler     :: unicode_width    :: UnicodeWidth ;

fn main()
{
	let example = "\
Memory safety without garbage collection. \
Concurrency without data races. \
Zero-cost abstractions.\
";

	let mut prev_lines = vec![];

	let c       = hyphenation::load( Language::English_US ).unwrap();
	let hyph    = Box::new( Hyphenator{ priority: 0, corpus: c, glue: "-\n".to_string() } );

	let xi      = Box::new( Xi{ priority: 0 } );

	let mut wrapper = Wrapper::new( 15, vec![ hyph, xi ], vec![], UnicodeWidth, false ).unwrap();


	for width in 15..60
	{
		wrapper.set_width( width ).expect( "need non-zero width" );

		if let Ok( s ) = wrapper.wrap( example )
		{
			let lines: Vec<_> = s.lines().map( |slice| slice.to_string() ).collect();

			if lines != prev_lines
			{
				let title = format!( " Width: {} ", width );

				println!( ".{:-^1$}.", title, width + 2 );

				for line in &lines
				{
					println!( "| {:1$} |", line, width );
				}

				prev_lines = lines;
			}
		}
	}
}
