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
	pub priority : WidthOffset           ,
	pub width    : Option< WidthOffset > ,
	pub enabled  : bool                  ,
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
			start    : ByteOffset ( start    ) ,
			end      : ByteOffset ( end      ) ,
			priority : WidthOffset( priority ) ,
			glue     : "\n".to_string()        ,
			mandatory: false                   ,
			enabled  : true                    ,
			width    : None                    ,
		}
	}


	// We substract the width of the glue, so that if two splitpoints would otherwise have the same score,
	// the one that doesn't need eg. hyphens wins.
	//
	pub fn score< Ruler: TextWidth >( &self, ruler: &Ruler ) -> WidthOffset
	{
		if self.width.is_none() {

			panic!( "Cannot calculate the score of a SplitPoint before setting it's width." ); }


		self.width.unwrap() + self.priority - ruler.measure( &self.glue )
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
	fn eq( &self, other: &Self ) -> bool
	{
		self.start     == other.start     &&
		self.end       == other.end       &&
		self.priority  == other.priority  &&
		self.glue      == other.glue      &&
		self.enabled   == other.enabled   &&
		self.mandatory == other.mandatory &&
		self.width     == other.width
	}
}


#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn constructor()
	{
		let s = SplitPoint::new( 1, 2, 3 );

		assert_eq!( s.start    , ByteOffset ( 1 ) );
		assert_eq!( s.end      , ByteOffset ( 2 ) );
		assert_eq!( s.priority , WidthOffset( 3 ) );
		assert_eq!( s.glue     , "\n".to_string() );
		assert_eq!( s.mandatory, false            );
		assert_eq!( s.enabled  , true             );
	}


	#[test]
	fn score()
	{
		let mut s = SplitPoint::new( 1, 2, 3 );

		s.width = Some( WidthOffset( 6 ) );
		s.glue  = "-".to_string()         ;

		assert_eq!( s.score( &ruler::unicode_width::UnicodeWidth ), WidthOffset ( 8 ) );

	}


	#[test] fn equal                          () { assert_eq!( SplitPoint::new( 3, 4, 0 ), SplitPoint::new( 3, 4, 0 ) ); }
	#[test] fn equal_should_have_same_start   () { assert_ne!( SplitPoint::new( 2, 4, 0 ), SplitPoint::new( 3, 4, 0 ) ); }
	#[test] fn equal_should_have_same_end     () { assert_ne!( SplitPoint::new( 3, 5, 0 ), SplitPoint::new( 3, 4, 0 ) ); }
	#[test] fn equal_should_have_same_priority() { assert_ne!( SplitPoint::new( 3, 5, 0 ), SplitPoint::new( 3, 5, 5 ) ); }


	#[test]
	fn equal_should_have_same_glue()
	{
		let     s = SplitPoint::new( 3, 5, 0 );
		let mut t = SplitPoint::new( 3, 5, 0 );

		t.glue = "0".to_string();

		assert_ne!( s, t );
	}


	#[test]
	fn equal_should_have_same_mandatory()
	{
		let     s = SplitPoint::new( 3, 5, 0 );
		let mut t = SplitPoint::new( 3, 5, 0 );

		t.mandatory = true;

		assert_ne!( s, t );
	}


	#[test]
	fn equal_should_have_same_enabled()
	{
		let     s = SplitPoint::new( 3, 5, 0 );
		let mut t = SplitPoint::new( 3, 5, 0 );

		t.enabled = false;

		assert_ne!( s, t );
	}


	// This is questionable, but for now mainly when we compare splitpoints it's in unit tests for generators. It probably
	// doesn't make much sense to compare splitpoints from different strings, so let's say width needs to be the same.
	//
	#[test]
	fn equal_should_have_same_width()
	{
		let mut s = SplitPoint::new( 3, 5, 0 );
		let mut t = SplitPoint::new( 3, 5, 0 );

		s.width = Some( WidthOffset( 4 ) );
		t.width = Some( WidthOffset( 3 ) );

		assert_ne!( s, t );
	}

}
