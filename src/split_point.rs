use std::cmp::Ordering;
use std::ops::Range;

use super::offset::ByteOffset;


#[ derive( Eq, Clone, Debug ) ]
//
pub struct SplitPoint<'a>
{
	pub span     : Range < ByteOffset > ,
	pub glue     : &'a str              ,
	pub mandatory: bool                 ,
}



impl<'a> Ord for SplitPoint<'a>
{
	fn cmp( &self, other: &Self ) -> Ordering
	{
		if self.span.start.cmp( &other.span.start ) == Ordering::Equal
		{
			self.span.end.cmp( &other.span.end )
		}

		else
		{
			self.span.start.cmp( &other.span.start )
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
		self.span == other.span
	}
}
