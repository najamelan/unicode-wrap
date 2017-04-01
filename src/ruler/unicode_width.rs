use super::*;
use self::TextWidth;
use self::unicode_width_crate::UnicodeWidthStr;

pub struct UnicodeWidth;

impl TextWidth for UnicodeWidth
{
	fn measure( &self, text: &str ) -> usize
	{
		text.width()
	}

}
