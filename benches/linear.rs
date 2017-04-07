#![feature(test)]

// The benchmarks here verify that the complexity grows as O(*n*)
// where *n* is the size of the text to be wrapped.

extern crate test;
extern crate hyphenation as hyphenation_crate;
extern crate unicode_wrap;

use hyphenation_crate::Language;

use unicode_wrap::*;

use generator :: hyphenation      :: Hyphenator   ;
use generator :: unicode_standard :: Xi           ;
use ruler     :: unicode_width    :: UnicodeWidth ;

use test::Bencher;

const LINE_LENGTH: usize = 60;

fn lorem_ipsum(length: usize) -> &'static str {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas feugiat non mi \
                rutrum consectetur. Nulla iaculis luctus ex suscipit posuere. Sed et tellus quis \
                elit volutpat pretium. Sed faucibus purus vitae feugiat tincidunt. Nulla \
                malesuada interdum tempus. Proin consectetur malesuada magna, id suscipit enim \
                tempus in. Sed sollicitudin velit tortor, quis condimentum nisl vulputate \
                lobortis. Curabitur id lectus arcu. Nullam quis aliquam nisi. Vestibulum quam \
                enim, elementum vel urna scelerisque, ultricies cursus urna. Mauris vestibulum, \
                augue non posuere viverra, risus tortor iaculis augue, eget convallis metus nisl \
                vestibulum nisi. Aenean auctor dui vel aliquet sagittis. Aliquam quis enim \
                mauris. Nunc eu leo et orci euismod bibendum vel eu tortor. Nam egestas volutpat \
                ex, a turpis duis.";
    text.split_at(length).0
}


fn run( size: usize, b: &mut Bencher, hyphenate: bool )
{
    let c    = hyphenation_crate::load( Language::Latin ).unwrap();
    let hyph = Box::new( Hyphenator{ priority: 0, corpus: c, glue: "-\n".to_string() } );
    let xi   = Box::new( Xi{ priority: 0 } );
    let text = lorem_ipsum( size );

    let mut gen: Vec< Box<Generate> > = vec![ xi ];

    if hyphenate { gen.push( hyph ) }

    let w = Wrapper::new( LINE_LENGTH, gen, vec![], UnicodeWidth, false ).expect( "Width should not be zero" );

    b.iter( || w.wrap( text ) );
}

#[bench] fn lorem_100            ( b: &mut Bencher ) { run( 100, b, false ); }
#[bench] fn lorem_200            ( b: &mut Bencher ) { run( 200, b, false ); }
#[bench] fn lorem_400            ( b: &mut Bencher ) { run( 400, b, false ); }
#[bench] fn lorem_800            ( b: &mut Bencher ) { run( 800, b, false ); }

#[bench] fn hyphenation_lorem_100( b: &mut Bencher ) { run( 100, b, true  ); }
#[bench] fn hyphenation_lorem_200( b: &mut Bencher ) { run( 200, b, true  ); }
#[bench] fn hyphenation_lorem_400( b: &mut Bencher ) { run( 400, b, true  ); }
#[bench] fn hyphenation_lorem_800( b: &mut Bencher ) { run( 800, b, true  ); }
