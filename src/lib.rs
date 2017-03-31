
// # unicode-textwrap RFC
//
// So far this proposes an algorithm that is capable of wrapping one line. Neither is it expected to deal with existing newline characters or
// to find the ideal split point for justifying paragraphs.
//
// - use generators    to generate split points.
// - use filters       to forbid certain split points.
// - use configuration to decide which generators and filters to use and with which priorities.
// - use ruler         to determine the width of a portion of text, so it can be compared to the maximum allowed/desired width for a line.
//
// ## Generators
//
// Generators propose split points. They should propose an endline character sequence to be added at the split point (eg. a hyphen
// for hyphenation, a backslash for a multiline string algorithm for programming code). They would also indicate characters that
// should be removed were a split actually be used (eg. remove the space on line split).
//
// - xi-unicode line breaking (unicode standard line breaking algorithm)
// - hyphenation
// - context specific (how to break a long url?)
// - ...
//
// ## Filters
//
// Filters forbid split points.
//
// - grapheme cluster enforcement, might not be necessary if generators guarantee to only propose split points on cluster boundaries.
// - language specific restrictions, such as don't break on the spaces in "« French ! »", even if they aren't encoded as non-breaking spaces.
// - context specific (eg. don't wrap text in <pre> tags for html)
// - ...
//
// Different priorities could be given to different generators and filters, and emergency implementations could be used in case
// no valid split point is found within the allowed/desired width. Eg. do we unpeel restrictions imposed by filters one by one until
// we find a split point within the allowed width?
//
// ## Ruler
//
// A ruler is an object that implements the TextWidth trait, and that will allow us to compare the width of a portion of text to the
// maximum allowed/desired width. unicode-width can be used for monospaced text, but in other situations one might want to consider the
// width in pixels or picas for example.
//
// ## Configuration:
//
// - which generators to use, with which priority? Eg. do we want to avoid hyphenation yet allow it if necessary?
// - which filters to use and do they pose a hard or a soft restriction?
// - does the desired width pose a hard or a soft limit, which filters can be ignored if needed and by how much can we exceed it?
// - allow plugging in logic with callbacks, much like ruby methods can take blocks?
//
// ## Extras
//
// Extra functionality is envisionable (eg. longest_word method which will return the minimum width of the biggest portion of text after
// taking into account all split points. This will be effectively the shortest possible width to which the text can be fitted).
//
// Naja Melan


extern crate hyphenation                                ;
extern crate unicode_width        as unicode_width_crate;
extern crate unicode_segmentation                       ;
extern crate xi_unicode                                 ;

pub mod wrapper    ;
pub mod generator  ;
pub mod split_point;
pub mod offset     ;
pub mod util       ;
pub mod ruler      ;


pub use self::split_point::SplitPoint;
pub use self::offset::ByteOffset;
pub use self::offset::WidthOffset;
pub use self::wrapper::Wrapper;



// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//     }
// }
