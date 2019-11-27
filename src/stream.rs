#![allow(dead_code)]

use std::fs;
use std::io::BufReader;
use std::path::Path;
use utf8_chars::BufReadCharsExt;

pub struct Stream<A: 'static>(Box<dyn FnOnce() -> Option<(A, Stream<A>)> + 'static>);

impl<A: 'static> Stream<A> {
    pub fn next(&mut self) -> Option<A> {
        let original = std::mem::replace(&mut self.0, Box::new(|| None));
        match original() {
            Some((next_element, next_stream)) => {
                self.0 = next_stream.0;
                Some(next_element)
            }
            None => None,
        }
    }

    pub fn new<F: FnMut() -> Option<A> + 'static>(mut function: F) -> Stream<A> {
        Stream(Box::new(move || match function() {
            Some(next) => Some((next, Stream::new(function))),
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

    pub fn filter<F: FnMut(&A) -> bool + 'static>(mut self, mut function: F) -> Stream<A> {
        Stream::new(move || loop {
            match self.next() {
                Some(a) if function(&a) => return Some(a),
                Some(_) => {}
                None => return None,
            }
        })
    }

    pub fn fold<Accumulator, F: FnMut(Accumulator, A) -> Accumulator>(
        self,
        initial: Accumulator,
        mut function: F,
    ) -> Accumulator {
        let mut accumulator = initial;
        for a in self {
            accumulator = function(accumulator, a);
        }
        accumulator
    }

    pub fn flat_map<B, Next: FnMut(A) -> Stream<B> + 'static>(self, next: Next) -> Stream<B> {
        self.map(next).flatten()
    }

    pub fn push(&mut self, head: A) {
        let original = std::mem::replace(&mut self.0, Box::new(|| None));
        self.0 = Box::new(move || Some((head, Stream(original))));
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
                self.push(a.clone());
                Some(a)
            }
            None => None,
        }
    }

    pub fn replicate(element: A, n: u32) -> Stream<A> {
        let mut counter = 0;
        Stream::new(move || {
            if counter < n {
                counter += 1;
                Some(element.clone())
            } else {
                None
            }
        })
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

#[macro_export]
macro_rules! stream {
    ($($x:expr),*) => {
        crate::stream::Stream::from(vec![$($x),*].into_iter())
    };
    ($($x:expr,)*) => {
        stream![$($x),*]
    };
    ($element:expr; $n:expr) => {
        crate::stream::Stream::replicate($element, $n)
    };
}

#[cfg(test)]
mod stream {
    use super::*;

    impl<A> Stream<A> {
        pub fn to_vec(self) -> Vec<A> {
            self.into_iter().collect()
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn allows_to_convert_from_iterator() {
            let iter = vec![1, 2, 3].into_iter();
            let from_next = Stream::from(iter);
            assert_eq!(from_next.to_vec(), vec![1, 2, 3]);
        }

        #[test]
        fn allows_to_convert_into_iterator() {
            let stream = stream!(1, 2, 3).into_iter();
            assert_eq!(stream.collect::<Vec<_>>(), vec![1, 2, 3]);
        }
    }

    mod stream_macro {
        use super::*;

        #[test]
        fn allows_to_convert_from_elements() {
            let stream: Stream<i32> = stream![1, 2, 3];
            assert_eq!(stream.to_vec(), vec![1, 2, 3]);
        }

        #[test]
        fn allows_to_create_empty_streams() {
            let stream: Stream<i32> = stream![];
            assert_eq!(stream.to_vec(), vec![]);
        }

        #[test]
        fn allows_to_trailing_commas() {
            let stream: Stream<i32> = stream![1, 2, 3,];
            assert_eq!(stream.to_vec(), vec![1, 2, 3]);
        }

        #[test]
        fn allows_to_replicate_a_given_element() {
            let stream: Stream<i32> = stream![42; 3];
            assert_eq!(stream.to_vec(), vec![42, 42, 42]);
        }
    }

    #[test]
    fn map_works() {
        let from_next: Stream<i32> = stream![1, 2, 3];
        let mapped = from_next.map(|x| x.pow(2));
        assert_eq!(vec![1, 4, 9], mapped.to_vec());
    }

    #[test]
    fn filter_works() {
        let stream = Stream::from(1..6).filter(|x| x % 2 == 1);
        assert_eq!(stream.to_vec(), vec![1, 3, 5]);
    }

    #[test]
    fn fold_works() {
        let sum = Stream::from(1..6).fold(0, |sum: i32, a| sum + a);
        assert_eq!(sum, 15);
    }

    #[test]
    fn flatten_works() {
        let flattened = stream!["foo", "bar"]
            .map(|x| Stream::from(x.chars()))
            .flatten();
        assert_eq!(vec!['f', 'o', 'o', 'b', 'a', 'r'], flattened.to_vec());
    }

    #[test]
    fn flatmap_works() {
        let stream = stream!["foo", "bar"].flat_map(|x| Stream::from(x.chars()));
        assert_eq!(vec!['f', 'o', 'o', 'b', 'a', 'r'], stream.to_vec());
    }

    #[test]
    fn push_works() {
        let mut stream = stream!["bar", "baz"].map(|x| x.to_string());
        stream.push("foo".to_string());
        assert_eq!(vec!["foo", "bar", "baz"], stream.to_vec());
    }

    mod peek {
        use super::*;

        #[test]
        fn peek_works() {
            let mut stream = stream!["foo", "bar"].map(|x| x.to_string());
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
