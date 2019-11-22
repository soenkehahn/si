#![allow(dead_code)]

pub struct Stream<'a> {
    first: Option<char>,
    iterator: Box<dyn Iterator<Item = char> + 'a>,
}

impl<'a> Stream<'a> {
    pub fn new<I: Iterator<Item = char> + 'a>(mut iterator: I) -> Stream<'a> {
        Stream {
            first: iterator.next(),
            iterator: Box::new(iterator),
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.first
    }
}

impl<'a> Iterator for Stream<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let result = self.first;
        self.first = self.iterator.next();
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn allows_to_stream_chars() {
        let mut stream = Stream::new("x".chars());
        assert_eq!(stream.next(), Some('x'));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn allows_to_peek_ahead() {
        let mut stream = Stream::new("x".chars());
        assert_eq!(stream.peek(), Some('x'));
    }

    #[test]
    fn peeking_does_not_consume_chars() {
        let mut stream = Stream::new("x".chars());
        stream.peek();
        assert_eq!(stream.next(), Some('x'));
    }

    #[test]
    fn peeking_works_twice() {
        let mut stream = Stream::new("ab".chars());
        stream.peek();
        assert_eq!(stream.peek(), Some('a'));
    }

    #[test]
    fn peeking_works_after_next() {
        let mut stream = Stream::new("ab".chars());
        stream.next();
        assert_eq!(stream.peek(), Some('b'));
    }
}
