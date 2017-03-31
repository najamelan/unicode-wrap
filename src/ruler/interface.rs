pub trait TextWidth
{
	fn measure( &self, text: &str ) -> usize;
}
