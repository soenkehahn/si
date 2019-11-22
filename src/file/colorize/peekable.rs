use crate::stream::Stream;

pub struct Peekable<A: 'static> {
    first: Option<A>,
    stream: Stream<A>,
}

impl<A: Copy> Peekable<A> {
    pub fn new(mut stream: Stream<A>) -> Peekable<A> {
        Peekable {
            first: stream.next(),
            stream,
        }
    }

    pub fn peek(&mut self) -> Option<A> {
        self.first
    }
}

impl<A: Copy> Iterator for Peekable<A> {
    type Item = A;

    fn next(&mut self) -> Option<A> {
        let result = self.first;
        self.first = self.stream.next();
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn allows_to_stream_chars() {
        let mut stream = Peekable::new(Stream::from_iterator("x".chars()));
        assert_eq!(stream.next(), Some('x'));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn allows_to_peek_ahead() {
        let mut stream = Peekable::new(Stream::from_iterator("x".chars()));
        assert_eq!(stream.peek(), Some('x'));
    }

    #[test]
    fn peeking_does_not_consume_chars() {
        let mut stream = Peekable::new(Stream::from_iterator("x".chars()));
        stream.peek();
        assert_eq!(stream.next(), Some('x'));
    }

    #[test]
    fn peeking_works_twice() {
        let mut stream = Peekable::new(Stream::from_iterator("ab".chars()));
        stream.peek();
        assert_eq!(stream.peek(), Some('a'));
    }

    #[test]
    fn peeking_works_after_next() {
        let mut stream = Peekable::new(Stream::from_iterator("ab".chars()));
        stream.next();
        assert_eq!(stream.peek(), Some('b'));
    }
}
