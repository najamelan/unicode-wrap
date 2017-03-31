use std::cmp::Ordering;

use super::ByteOffset;


#[ derive( Eq, Clone, Debug ) ]
//
pub struct SplitPoint<'a>
{
	pub start    : ByteOffset,
	pub end      : ByteOffset,
	pub glue     : &'a str   ,
	pub mandatory: bool      ,
}



impl<'a> Ord for SplitPoint<'a>
{
	fn cmp( &self, other: &Self ) -> Ordering
	{
		if self.start.cmp( &other.start ) == Ordering::Equal
		{
			self.end.cmp( &other.end )
		}

		else
		{
			self.start.cmp( &other.start )
		}
	}
}



impl<'a> PartialOrd for SplitPoint<'a>
{
	fn partial_cmp( &self, other: &Self ) -> Option< Ordering >
	{
		Some( self.cmp( other ) )
	}
}



impl<'a> PartialEq for SplitPoint<'a>
{
	fn eq( &self, other: &Self ) -> bool
	{
		self.start == other.start && self.end == other.end
	}
}
