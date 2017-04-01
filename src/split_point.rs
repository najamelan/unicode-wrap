use std::cmp::Ordering;

use super::*;


#[ derive( Eq, Clone, Debug ) ]
//
pub struct SplitPoint<'a>
{
	pub start    : ByteOffset            ,
	pub end      : ByteOffset            ,
	pub glue     : &'a str               ,
	pub mandatory: bool                  ,
	pub priority : usize                 ,
	pub width    : Option< WidthOffset > ,
}



impl<'a> SplitPoint<'a>
{
	pub fn score( &self ) -> usize
	{
		if self.width.is_none() {

			panic!( "Cannot calculate the score of a SplitPoint before setting it's width." ); }


		self.width.unwrap().0 + self.priority
	}
}



impl<'a> Ord for SplitPoint<'a>
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



impl<'a> PartialOrd for SplitPoint<'a>
{
	fn partial_cmp( &self, other: &Self ) -> Option< Ordering > {

		Some( self.cmp( other ) ) }

}



impl<'a> PartialEq for SplitPoint<'a>
{
	fn eq( &self, other: &Self ) -> bool {

		self.start == other.start && self.end == other.end }

}
