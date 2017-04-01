use std::collections::HashMap;
use std::ops::Add;
use std::ops::Sub;
// use std::ops::Deref;

#[ derive( PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Hash, Copy ) ]
//
pub struct ByteOffset ( pub usize );

#[ derive( PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Hash, Copy ) ]
//
pub struct WidthOffset( pub usize );



impl ByteOffset
{
	// Returns the width of the string up until this ByteOffset. If it falls within a grapheme cluster, the beginning
	// of the containing cluster is returned.
	//
	pub fn to_width( self, b2w: &HashMap < ByteOffset, WidthOffset > ) -> WidthOffset
	{
		let mut found = WidthOffset( 0 );
		let mut try   = self;

		loop
		{
			if b2w.contains_key( &try )
			{
				found = b2w[ &try ];
				break;
			}

			else if try.0 > 1
			{
				try = try - 1;
			}

			else { break }
		}

		found
	}
}



impl WidthOffset
{
	// Returns the ByteOffset of the string up until this width. If it falls within a grapheme cluster, the beginning
	// of the containing cluster is returned (yes, some clusters have a width of several columns).
	//
	pub fn to_bytes( self, w2b: &HashMap < WidthOffset, ByteOffset > ) -> ByteOffset
	{
		let mut found = ByteOffset( 0 );
		let mut try   = self;

		loop
		{
			if w2b.contains_key( &try )
			{
				found = w2b[ &try ];
				break;
			}

			else if try.0 > 1
			{
				try = try - 1;
			}

			else { break }
		}

		found
	}
}


impl Add for ByteOffset
{
	type Output = Self;

	fn add( self, other: Self ) -> Self
	{
		ByteOffset( self.0 + other.0 )
	}
}


impl Sub for ByteOffset
{
	type Output = Self;

	fn sub( self, other: Self ) -> Self
	{
		ByteOffset( self.0 - other.0 )
	}
}


impl Add<usize> for ByteOffset
{
	type Output = Self;

	fn add( self, other: usize ) -> Self
	{
		ByteOffset( self.0 + other )
	}
}


impl Sub<usize> for ByteOffset
{
	type Output = Self;

	fn sub( self, other: usize ) -> Self
	{
		ByteOffset( self.0 - other )
	}
}


// impl Deref for ByteOffset
// {
// 	type Target = usize;

// 	fn deref( &self ) -> &usize
// 	{
// 		&self.0
// 	}
// }


impl Add for WidthOffset
{
	type Output = Self;

	fn add( self, other: Self ) -> Self
	{
		WidthOffset( self.0 + other.0 )
	}
}


impl Sub for WidthOffset
{
	type Output = Self;

	fn sub( self, other: Self ) -> Self
	{
		WidthOffset( self.0 - other.0 )
	}
}


impl Add<usize> for WidthOffset
{
	type Output = Self;

	fn add( self, other: usize ) -> Self
	{
		WidthOffset( self.0 + other )
	}
}


impl Sub<usize> for WidthOffset
{
	type Output = Self;

	fn sub( self, other: usize ) -> Self
	{
		WidthOffset( self.0 - other )
	}
}


// impl Deref for WidthOffset
// {
// 	type Target = usize;

// 	fn deref( &self ) -> &usize
// 	{
// 		&self.0
// 	}
// }
