use std::collections::HashMap;

use super::*;

use generator::interface::Generator;
use ruler    ::interface::TextWidth;


pub struct Wrapper<'a, Ruler>
{
	pub width     : usize                                 ,
	pub generators: HashMap< usize, Vec< &'a Generator > >,
	pub ruler     : Ruler                                 ,
}


impl<'a, Ruler> Wrapper<'a, Ruler>

	where Ruler: TextWidth
{
	pub fn wrap_line( &self, line: &str ) -> Vec< String >
	{
		let mut splits: HashMap< usize, Vec< SplitPoint > > = HashMap::new();

		for ( prio, generators ) in &self.generators
		{
			for gen in generators
			{
				if splits.contains_key( prio )
				{
					splits.get_mut( prio ).unwrap().append( &mut gen.opportunities( line ) );
				}

				else
				{
					splits.insert( *prio, gen.opportunities( line ) );
				}
			}
		}

		println!( "splits: {:?}", splits );

		vec![ "".to_string() ]
	}
}




#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn tmp()
	{
		let inlist = &generator::unicode_standard::Xi as &generator::interface::Generator;
		let list = vec![ inlist ];

		let mut gens = HashMap::new();
		gens.insert( 1, list );

		let wrapper = Wrapper
		{
			width     : 3                                 ,
			generators: gens                              ,
			ruler     : ruler::unicode_width::UnicodeWidth,
		};

		wrapper.wrap_line( "ha ha ah" );
	}
}
