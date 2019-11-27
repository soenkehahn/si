use std::fs;
use std::io::BufReader;
use std::path::Path;
use utf8_chars::BufReadCharsExt;

pub struct Stream<A: 'static>(Box<dyn FnOnce() -> Option<(Stream<A>, A)> + 'static>);

impl<A: 'static> Stream<A> {
    pub fn next(&mut self) -> Option<A> {
        let original = std::mem::replace(&mut self.0, Box::new(|| None));
        match original() {
            Some((next_stream, next_element)) => {
                self.0 = next_stream.0;
                Some(next_element)
            }
            None => None,
        }
    }

    pub fn new<F: FnMut() -> Option<A> + 'static>(mut function: F) -> Stream<A> {
        Stream(Box::new(move || match function() {
            Some(next) => Some((Stream::new(function), next)),
            None => None,
        }))
    }

    pub fn empty() -> Stream<A> {
        Stream(Box::new(|| None))
    }

    pub fn map<B, F: FnMut(A) -> B + 'static>(mut self, mut function: F) -> Stream<B> {
        Stream::new(move || match self.next() {
            Some(x) => Some(function(x)),
            None => None,
        })
    }

    pub fn flat_map<B, Next: FnMut(A) -> Stream<B> + 'static>(self, next: Next) -> Stream<B> {
        self.map(next).flatten()
    }

    pub fn cons(&mut self, head: A) {
        let original = std::mem::replace(&mut self.0, Box::new(|| None));
        self.0 = Box::new(move || Some((Stream(original), head)));
    }
}

impl<A, I: Iterator<Item = A> + 'static> From<I> for Stream<A> {
    fn from(mut iterator: I) -> Self {
        Stream::new(move || iterator.next())
    }
}

impl<A> Stream<Stream<A>> {
    pub fn flatten(mut self) -> Stream<A> {
        let mut current = Stream::empty();
        Stream::new(move || loop {
            match current.next() {
                Some(a) => return Some(a),
                None => match self.next() {
                    Some(next_chunk) => {
                        current = next_chunk;
                    }
                    None => return None,
                },
            }
        })
    }
}

impl<A: Clone> Stream<A> {
    pub fn peek(&mut self) -> Option<A> {
        match self.next() {
            Some(a) => {
                self.cons(a.clone());
                Some(a)
            }
            None => None,
        }
    }
}

impl Stream<char> {
    pub fn read_utf8_file(file: &Path) -> Result<Stream<char>, std::io::Error> {
        let mut file = BufReader::new(fs::File::open(file)?);
        Ok(Stream::new(move || {
            file.read_char().expect("utf8 decoding error")
        }))
    }
}

impl<A> IntoIterator for Stream<A> {
    type Item = A;
    type IntoIter = StreamIterator<A>;
    fn into_iter(self) -> StreamIterator<A> {
        StreamIterator(self)
    }
}

pub struct StreamIterator<A: 'static>(Stream<A>);

impl<A> Iterator for StreamIterator<A> {
    type Item = A;

    fn next(&mut self) -> Option<A> {
        self.0.next()
    }
}

#[cfg(test)]
mod stream {
    use super::*;

    impl<A> Stream<A> {
        pub fn to_vec(self) -> Vec<A> {
            self.into_iter().collect()
        }
    }

    #[test]
    fn allows_to_iterate_over_a_vector() {
        let vec = vec![1, 2, 3];
        let mut vec_iter = vec.into_iter();
        let stream = Stream::new(move || vec_iter.next());
        assert_eq!(vec![1, 2, 3], stream.to_vec());
    }

    #[test]
    fn allows_to_convert_from_iterator() {
        let from_next = Stream::from(vec![1, 2, 3].into_iter());
        assert_eq!(vec![1, 2, 3], from_next.to_vec());
    }

    #[test]
    fn map_works() {
        let from_next: Stream<i32> = Stream::from(vec![1, 2, 3].into_iter());
        let mapped = from_next.map(|x| x.pow(2));
        assert_eq!(vec![1, 4, 9], mapped.to_vec());
    }

    #[test]
    fn flatten_works() {
        let from_next =
            Stream::from(vec!["foo", "bar"].into_iter()).map(|x| Stream::from(x.chars()));
        assert_eq!(
            vec!['f', 'o', 'o', 'b', 'a', 'r'],
            from_next.flatten().to_vec()
        );
    }

    #[test]
    fn flatmap_works() {
        let stream =
            Stream::from(vec!["foo", "bar"].into_iter()).flat_map(|x| Stream::from(x.chars()));
        assert_eq!(vec!['f', 'o', 'o', 'b', 'a', 'r'], stream.to_vec());
    }

    #[test]
    fn cons_works() {
        let mut stream = Stream::from(vec!["bar", "baz"].into_iter().map(|x| x.to_string()));
        stream.cons("foo".to_string());
        assert_eq!(vec!["foo", "bar", "baz"], stream.to_vec());
    }

    mod peek {
        use super::*;

        #[test]
        fn peek_works() {
            let mut stream = Stream::from(vec!["foo", "bar"].into_iter().map(|x| x.to_string()));
            assert_eq!(stream.peek(), Some("foo".to_string()));
            assert_eq!(vec!["foo", "bar"], stream.to_vec());
        }

        #[test]
        fn allows_to_peek_ahead() {
            let mut stream = Stream::from("x".chars());
            assert_eq!(stream.peek(), Some('x'));
        }

        #[test]
        fn peeking_does_not_consume_chars() {
            let mut stream = Stream::from("x".chars());
            stream.peek();
            assert_eq!(stream.next(), Some('x'));
        }

        #[test]
        fn peeking_works_twice() {
            let mut stream = Stream::from("ab".chars());
            stream.peek();
            assert_eq!(stream.peek(), Some('a'));
        }

        #[test]
        fn peeking_works_after_next() {
            let mut stream = Stream::from("ab".chars());
            stream.next();
            assert_eq!(stream.peek(), Some('b'));
        }
    }
}
