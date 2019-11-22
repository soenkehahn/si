use std::fs;
use std::io::BufReader;
use std::path::Path;
use utf8_chars::BufReadCharsExt;

pub struct Stream<A: 'static>(Box<dyn FnMut() -> Option<A> + 'static>);

impl<A: 'static> Stream<A> {
    pub fn next(&mut self) -> Option<A> {
        (self.0)()
    }

    pub fn new<F: FnMut() -> Option<A> + 'static>(function: F) -> Stream<A> {
        Stream(Box::new(function))
    }

    pub fn from_iterator<I: std::iter::Iterator<Item = A> + 'static>(mut input: I) -> Stream<A> {
        Stream::new(move || input.next())
    }

    pub fn empty() -> Stream<A> {
        Stream(Box::new(|| None))
    }

    pub fn map<B, F: Fn(A) -> B + 'static>(mut self, function: F) -> Stream<B> {
        Stream(Box::new(move || match self.next() {
            Some(x) => Some(function(x)),
            None => None,
        }))
    }

    pub fn flat_map<B, Next: Fn(A) -> Stream<B> + 'static>(self, next: Next) -> Stream<B> {
        self.map(next).flatten()
    }
}

impl<A> Stream<Stream<A>> {
    pub fn flatten(mut self) -> Stream<A> {
        let mut current = Stream::empty();
        Stream(Box::new(move || loop {
            match current.next() {
                Some(a) => return Some(a),
                None => match self.next() {
                    Some(next_chunk) => {
                        current = next_chunk;
                    }
                    None => return None,
                },
            }
        }))
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
        let from_next = Stream::from_iterator(vec![1, 2, 3].into_iter());
        assert_eq!(vec![1, 2, 3], from_next.to_vec());
    }

    #[test]
    fn map_works() {
        let from_next: Stream<i32> = Stream::from_iterator(vec![1, 2, 3].into_iter());
        let mapped = from_next.map(|x| x.pow(2));
        assert_eq!(vec![1, 4, 9], mapped.to_vec());
    }

    #[test]
    fn flatten_works() {
        let from_next = Stream::from_iterator(vec!["foo", "bar"].into_iter())
            .map(|x| Stream::from_iterator(x.chars()));
        assert_eq!(
            vec!['f', 'o', 'o', 'b', 'a', 'r'],
            from_next.flatten().to_vec()
        );
    }

    #[test]
    fn flatmap_works() {
        let stream = Stream::from_iterator(vec!["foo", "bar"].into_iter())
            .flat_map(|x| Stream::from_iterator(x.chars()));
        assert_eq!(vec!['f', 'o', 'o', 'b', 'a', 'r'], stream.to_vec());
    }
}
