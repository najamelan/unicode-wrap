

extern crate hyphenation as hyphenation_crate;
extern crate unicode_wrap;

use hyphenation_crate::Language;
use unicode_wrap::Wrapper;
use unicode_wrap::generator::hyphenation::Hyphenator;
use unicode_wrap::generator::unicode_standard::Xi;
use unicode_wrap::ruler::unicode_width::UnicodeWidth;

fn main()
{
	let example = "\
Memory safety without garbage collection. \
Concurrency without data races. \
Zero-cost abstractions.\
";

	let mut prev_lines = vec![];

	let c       = hyphenation_crate::load( Language::English_US ).unwrap();
	let hyph    = Hyphenator{ priority: 0, corpus: &c, glue: "-".to_string() };

	let xi      = Xi{ priority: 0 };

	let mut wrapper = Wrapper
	{
		width     : 15                 ,
		generators: vec![ &hyph, &xi ] ,
		ruler     : UnicodeWidth       ,
		filters   : vec![]             ,
	};

	for width in 15..60
	{
		wrapper.width = width;

		let lines = wrapper.wrap_line( example );

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
