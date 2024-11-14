use std::error::Error;

use std::io::prelude::*;
use std::io::Seek;
use std::io::SeekFrom;

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

#[derive(Debug)]
/// Represents ID3v1 tags
/// Based on: https://id3.org/ID3v1
pub struct ID3v1 {
    title: String,
    artist: String,
    album: String,
    year: String,
    comment: String,
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

impl ID3v1 {
    /// Creates ID3V1 struct from a readable source
    pub fn read<T: Seek + Read>(source: &mut T) -> Result<ID3v1, ReadError> {
        source.seek(SeekFrom::End(-128))?;

        let mut tag = [0; 3];
        source.read_exact(&mut tag)?;
        if &tag != b"TAG" {
            return Err(ReadError::ID3);
        }

        let title = ID3v1::get_field_string(30, source)?;
        let artist = ID3v1::get_field_string(30, source)?;
        let album = ID3v1::get_field_string(30, source)?;
        let year = ID3v1::get_field_string(4, source)?;
        let comment = ID3v1::get_field_string(30, source)?;

        let genre = match source.bytes().next() {
            Some(res) => res?,
            None => return Err(ReadError::ID3),
        };

        Ok(ID3v1 {
            title,
            artist,
            album,
            year,
            comment,
            genre,
        })
    }

    /// Converts an ID3V1 field into a String
    fn get_field_string<T: Seek + Read>(
        field_length: usize,
        source: &mut T,
    ) -> std::io::Result<String> {
        let mut buff = Vec::with_capacity(field_length);
        // https://users.rust-lang.org/t/read-until-buffer-is-full-or-eof/90184
        source.take(field_length as u64).read_to_end(&mut buff)?;
        // https://stackoverflow.com/questions/28169745/what-are-the-options-to-convert-iso-8859-1-latin-1-to-a-string-utf-8
        let field = buff.iter().map(|&c| c as char).collect();
        Ok(field)
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
