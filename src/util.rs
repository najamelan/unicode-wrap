// Unicode codepoints with the Ws property. These exclude no-break spaces.
//
const WHITESPACE: &'static [char] = &
[
	'\u{0009}' , // TAB STOP
	'\u{0020}' , // SPACE
	'\u{1680}' , // OGHAM SPACE MARK
	'\u{2000}' , // EN QUAD
	'\u{2001}' , // EM QUAD
	'\u{2002}' , // EN SPACE
	'\u{2003}' , // EM SPACE
	'\u{2004}' , // THREE-PER-EM
	'\u{2005}' , // FOUR-PER-EM
	'\u{2006}' , // SIX-PER-EM
	'\u{2007}' , // FIGURE SPACE
	'\u{2008}' , // PUNCTUATION SPACE
	'\u{2009}' , // THIN SPACE
	'\u{200A}' , // HAIR SPACE
	'\u{2028}' , // LINE SEPARATOR
	'\u{205F}' , // MEDIUM MATHEMATICAL SPACE
	'\u{3000}' , // IDEOGRAPHIC SPACE
];

pub fn char_is_whitespace( c: &char ) -> bool
{
	WHITESPACE.contains( &c )
}

// Unicode codepoints for characters that will consitute a hard break.
//
// const ENDLINE: &'static [char] = &
// [
// 	'\u{000A}' , // LINE FEED
// 	'\u{000B}' , // LINE TABULATION (VT)
// 	'\u{000C}' , // FORM FEED (FF)
// 	'\u{000D}' , // CARRIAGE RETURN (CR)
// 	'\u{0085}' , // NEXT LINE (NEL)
// 	'\u{2028}' , // LINE SEPARATOR (LS)
// 	'\u{2029}' , // PARAGRAPH SEPARATOR (PS)
// ];



#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn char_is_whitespace_space()
	{
		assert!( char_is_whitespace( &' ' ) );
	}
}
