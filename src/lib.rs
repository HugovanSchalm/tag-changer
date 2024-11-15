use std::error::Error;

use std::io::prelude::*;
use std::io::Seek;
use std::io::SeekFrom;

use std::convert::From;

#[derive(Debug)]
/// Differentiate between IO error and an error in reading the ID3 tags.
pub enum ReadError {
    IO(std::io::Error),
    ID3,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ReadError {}

impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> Self {
        ReadError::IO(err)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
/// The encoding of ID3v1 text is base on
/// ISO 8859-1 (https://www.wikipedia.org/wiki/ISO_8859-1)
struct ISO_8859_1(String);

impl From<&[u8]> for ISO_8859_1 {
    fn from(value: &[u8]) -> Self {
        // https://stackoverflow.com/questions/28169745/what-are-the-options-to-convert-iso-8859-1-latin-1-to-a-string-utf-8
        ISO_8859_1(value.iter().map(|&c| c as char).collect())
    }
}

impl From<&str> for ISO_8859_1 {
    fn from(value: &str) -> Self {
        ISO_8859_1(String::from(value))
    }
}

impl std::fmt::Display for ISO_8859_1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
/// Represents ID3v1 tags
/// Based on: https://id3.org/ID3v1
pub struct ID3v1 {
    title: ISO_8859_1,
    artist: ISO_8859_1,
    album: ISO_8859_1,
    year: ISO_8859_1,
    comment: ISO_8859_1,
    genre: u8,
}

impl std::fmt::Display for ID3v1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\
Song title: {}
Artist: {}
Album: {}
Year: {}
Comment: {}
Genre: {}\
            ",
            self.title,
            self.artist,
            self.album,
            self.year,
            self.comment,
            self.get_genre_str()
        )
    }
}

impl TryFrom<Vec<u8>> for ID3v1 {
    type Error = ReadError;

    fn try_from(value: Vec<u8>) -> Result<Self, ReadError> {
        if value.len() != 128 {
            return Err(ReadError::ID3);
        }

        let ISO_8859_1(tag) = ISO_8859_1::from(&value[0..=2]);
        println!("{}", tag);
        if tag != "TAG" {
            return Err(ReadError::ID3);
        }

        let title = ISO_8859_1::from(&value[3..=32]);
        let artist = ISO_8859_1::from(&value[33..=62]);
        let album = ISO_8859_1::from(&value[63..=92]);
        let year = ISO_8859_1::from(&value[93..=96]);
        let comment = ISO_8859_1::from(&value[97..=126]);
        let genre = value[127];

        Ok(ID3v1 {
            title,
            artist,
            album,
            year,
            comment,
            genre,
        })
    }
}

impl From<ID3v1> for Vec<u8> {
    fn from(tags: ID3v1) -> Self {
        let mut result = Vec::with_capacity(128);
        for &c in b"TAG" {
            result.push(c);
        }

        let text_fields = [
            (tags.title, 30),
            (tags.artist, 30),
            (tags.album, 30),
            (tags.year, 4),
            (tags.comment, 30),
        ];

        for field in text_fields {
            println!("{}", field.0 .0);
            for c in field.0 .0.bytes() {
                println!("{}", c);
                result.push(c);
            }

            for _ in field.0 .0.len()..field.1 {
                result.push(0)
            }
        }

        result.push(tags.genre);

        result
    }
}

impl ID3v1 {
    /// Creates ID3V1 struct from a readable source
    pub fn read<T: Seek + Read>(source: &mut T) -> Result<ID3v1, ReadError> {
        source.seek(SeekFrom::End(-128))?;

        let mut buff = vec![0; 128];
        // https://users.rust-lang.org/t/read-until-buffer-is-full-or-eof/90184
        source.read_exact(&mut buff)?;

        println!("{:?}", buff);

        ID3v1::try_from(buff)
    }

    fn get_contents_without_tag<T: Read + Write + Seek>(from: &mut T) -> Result<Vec<u8>, std::io::Error> {
        let end_position = if ID3v1::read(from).is_ok() {
                println!("Has tag");
                from.seek(SeekFrom::End(-128)).unwrap()
            } else {
                println!("Has no tag");
                from.seek(SeekFrom::End(0)).unwrap()
            };
        
        from.seek(SeekFrom::Start(0))?;
        let mut buff = vec![0; end_position as usize];
        from.read_exact(&mut buff)?;
        Ok(buff)
    }

    fn write<T: Read + Write + Seek>(self, destination: &mut T) -> Result<(), std::io::Error> {
        let mut contents = Self::get_contents_without_tag(destination)?;
        contents.append(&mut self.into());
        destination.seek(SeekFrom::Start(0))?;
        destination.write_all(&contents)?;
        Ok(())
    }

    /// Converts a genre byte into a str according to https://en.wikipedia.org/wiki/List_of_ID3v1_genres
    fn get_genre_str(&self) -> &str {
        match &self.genre {
            0 => "Blues",
            1 => "Classic rock",
            2 => "Country",
            3 => "Dance",
            4 => "Disco",
            5 => "Funk",
            6 => "Grunge",
            7 => "Hip-hop",
            8 => "Jazz",
            9 => "Metal",
            10 => "New age",
            11 => "Oldies",
            12 => "Other",
            13 => "Pop",
            14 => "Rythm and blues",
            15 => "Rap",
            16 => "Reggae",
            17 => "Rock",
            18 => "Techno",
            19 => "Industrial",
            20 => "Alternative",
            21 => "Ska",
            22 => "Death Metal",
            23 => "Soundtrack",
            25 => "Euro-techno",
            26 => "Ambient",
            27 => "Trip-hop",
            28..=191 => todo!("the requested genre was not yet implemented"),
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    struct TestFile {
        cursor_position: usize,
        contents: Vec<u8>,
    }

    impl Read for TestFile {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let amount = (self.contents.len() - self.cursor_position).min(buf.len());
            for i in 0..amount {
                buf[i] = self.contents[self.cursor_position];
                self.cursor_position += 1;
            }
            Ok(amount)
        }
    }

    impl Write for TestFile {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            for &byte in buf {
                if self.cursor_position >= self.contents.len() {
                    self.contents.push(byte);
                } else {
                    self.contents[self.cursor_position] = byte;
                }

                self.cursor_position += 1;
            }
            return Ok(buf.len());
        }

        fn flush(&mut self) -> std::io::Result<()> {
            return Ok(());
        }
    }

    impl Seek for TestFile {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.cursor_position = match pos {
                SeekFrom::Current(offset) => {
                    let result = self.cursor_position as i64 + offset;
                    if result < 0 {
                        return Err(std::io::Error::from(std::io::ErrorKind::AddrNotAvailable));
                    }
                    result as usize
                }
                SeekFrom::Start(offset) => offset as usize,
                SeekFrom::End(offset) => {
                    let result = self.contents.len() as i64 + offset;
                    if result < 0 {
                        return Err(std::io::Error::from(std::io::ErrorKind::AddrNotAvailable));
                    }
                    result as usize
                }
            };
            Ok(self.cursor_position as u64)
        }
    }

    impl TestFile {
        fn new() -> Self {
            TestFile {
                cursor_position: 0,
                contents: vec![],
            }
        }
    }

    #[test]
    fn basic_write() {
        let mut test_file = TestFile::new();
        let title = b"testsong";
        let artist = b"testartist";
        let album = b"testalbum";
        let year = b"2024";
        let comment = b"testcomment";

        let tags = ID3v1 {
            title: title[0..].into(),
            artist: artist[0..].into(),
            album: album[0..].into(),
            year: year[0..].into(),
            comment: comment[0..].into(),
            genre: 5,
        };
        println!("{:?}", tags);
        tags.write(&mut test_file).unwrap();

        println!("{:#?}", test_file.contents);
        assert_eq!(test_file.contents.len(), 128);
        assert_eq!(&test_file.contents[0..=2], b"TAG");
        assert_eq!(test_file.contents[3..3 + title.len()], title[0..]);
        assert_eq!(
            test_file.contents[3 + title.len()..=32],
            vec!(0; 30 - title.len())
        );
        assert_eq!(test_file.contents[33..33 + artist.len()], artist[0..]);
        assert_eq!(
            test_file.contents[33 + artist.len()..=62],
            vec!(0; 30 - artist.len())
        );
        assert_eq!(test_file.contents[63..63 + album.len()], album[0..]);
        assert_eq!(
            test_file.contents[63 + album.len()..=92],
            vec!(0; 30 - album.len())
        );
        assert_eq!(test_file.contents[93..93 + year.len()], year[0..]);
        assert_eq!(
            test_file.contents[93 + year.len()..=96],
            vec!(0; 4 - year.len())
        );
        assert_eq!(test_file.contents[97..97 + comment.len()], comment[0..]);
        assert_eq!(
            test_file.contents[97 + comment.len()..=126],
            vec!(0; 30 - comment.len())
        );
        assert_eq!(test_file.contents[127], 5);
    }

    #[test]
    fn get_contents_too_short() {
        let mut test_file = TestFile {
            cursor_position: 0,
            contents: vec![1; 20],
        };

        ID3v1::get_contents_without_tag(&mut test_file).unwrap();
        assert_eq!(test_file.contents, vec![1; 20]);
    }

    #[test]
    fn get_contents_no_tag() {
        let mut test_file = TestFile {
            cursor_position: 0,
            contents: vec![1; 300],
        };

        ID3v1::get_contents_without_tag(&mut test_file).unwrap();
        assert_eq!(test_file.contents, vec![1; 300]);
    }

    #[test]
    fn get_contents() {
        let mut rng = rand::thread_rng();

        let mut test_file = TestFile {
            cursor_position: 0,
            contents: Vec::with_capacity(130),
        };

        test_file.contents.push(1);
        test_file.contents.push(2);
        test_file.contents.push(b'T');
        test_file.contents.push(b'A');
        test_file.contents.push(b'G');
        for _ in 0..125 {
            test_file.contents.push(rng.gen());
        }

        println!("{}", test_file.contents.len());

        let contents = ID3v1::get_contents_without_tag(&mut test_file).unwrap();

        assert_eq!(contents, vec![1, 2]);
    }
}
