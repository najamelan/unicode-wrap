use std::cmp::Ordering;

use super::*;


#[ derive( Eq, Clone, Debug ) ]
//
pub struct SplitPoint
{
	pub start    : ByteOffset            ,
	pub end      : ByteOffset            ,
	pub glue     : String                ,
	pub mandatory: bool                  ,
	pub priority : usize                 ,
	pub width    : Option< WidthOffset > ,
}



impl SplitPoint
{
	pub fn new
	(
		start    : usize,
		end      : usize,
		priority : usize,

	)  -> SplitPoint
	{
		SplitPoint
		{
			start    : ByteOffset( start ),
			end      : ByteOffset( end   ),
			priority : priority           ,
			glue     : "".to_string()     ,
			mandatory: false              ,
			width    : None               ,
		}
	}


	pub fn score( &self ) -> usize
	{
		if self.width.is_none() {

			panic!( "Cannot calculate the score of a SplitPoint before setting it's width." ); }


		self.width.unwrap().0 + self.priority
	}
}



impl Ord for SplitPoint
{
	fn cmp( &self, other: &Self ) -> Ordering
	{
		let first_choice =

				 (    self .priority + self .start.0   )
			.cmp( &( other.priority + other.start.0 ) )

		;


		if first_choice == Ordering::Equal {

			self.end.cmp( &other.end ) }


		else { first_choice }
	}
}



impl PartialOrd for SplitPoint
{
	fn partial_cmp( &self, other: &Self ) -> Option< Ordering > {

		Some( self.cmp( other ) ) }

}



impl PartialEq for SplitPoint
{
	fn eq( &self, other: &Self ) -> bool {

		self.start == other.start && self.end == other.end }

}
