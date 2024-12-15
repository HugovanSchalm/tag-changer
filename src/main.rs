use clap::{Arg, Command, Subcommand};
use std::{fs::File, path::PathBuf};

use tag_changer::{ID3v1, ReadError};

fn main() {
    let matches = Command::new("tag-changer")
        .author("Hugo van Schalm")
        .version("0.1.0")
        .subcommand(Command::new("view")
            .short_flag('v') 
            .long_flag("view")
            .arg(
                Arg::new("file")
                    .required(true)    
                    .value_parser(clap::builder::StringValueParser::new()),
            )
        )
        .subcommand(Command::new("edit")
            .short_flag('e') 
            .long_flag("edit")
            .arg(
                Arg::new("file")
                    .required(true)    
                    .value_parser(clap::builder::StringValueParser::new()),
            )
            .arg(
                Arg::new("title")
                    .short('t')
                    .long("title")
                    .value_parser(clap::builder::StringValueParser::new()),
            )
        )
        .get_matches();
    match matches.subcommand(){
        Some(("view", viewmatches)) => {
            let filestring: &String = viewmatches.get_one("file").unwrap();
            let mut file = open_file(filestring).unwrap();
            let tags = match ID3v1::read(&mut file) {
                Ok(tags) => tags,
                Err(ReadError::ID3) => panic!("Could not parse tags of file {}", filestring),
                Err(ReadError::IO(err)) => panic!("Could not open file: {}", err),
            };

            println!("{}", tags);
        }
        Some(("edit", editmatches)) => {
            let filestring: &String = editmatches.get_one("file").unwrap();
            let mut file = open_file(filestring).unwrap();
            let mut tag = ID3v1::default();
            if let Some(title) = editmatches.get_one::<String>("title"){
                tag.set_title(title);
            }
            tag.write(&mut file).unwrap();
        }
        _ => unreachable!("This subcommand should not exist"),
    }  
}

fn open_file(filestring: &String) -> std::io::Result<File>{
    let filepath = PathBuf::from(filestring);
    File::options()
        .write(true)
        .read(true)
        .open(filepath)
}