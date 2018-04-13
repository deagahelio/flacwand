#![feature(panic_info_message)]

extern crate metaflac;
extern crate argparse;

use std::io;
use std::io::prelude::*;
use std::panic;
use metaflac::Tag;
use argparse::{ArgumentParser, Store};

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        match panic_info.message() {
            Some(message) => eprintln!("Fatal Error: {}", message),
            None => eprintln!("Fatal Error: unknown"),
        };
    }));

    let mut command = "".to_owned();
    let mut file_path = "".to_owned();
    let mut field = "".to_owned();
    let mut value = "".to_owned();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Utility for modifying FLAC metadata");
        parser.refer(&mut file_path)
            .required()
            .add_argument("file", Store,
                          "target FLAC file");
        parser.refer(&mut command)
            .required()
            .add_argument("command", Store,
                          "command to run. SET, GET, SETUP and PRINT are available.");
        parser.refer(&mut field)
            .add_argument("field", Store,
                          "field used for the command, if one is needed");
        parser.refer(&mut value)
            .add_argument("value", Store,
                          "value used for the command, if one is needed");
        parser.parse_args_or_exit();
    }

    let mut tag = Tag::read_from_path(file_path).expect("Invalid path to FLAC file");

    {
        let vorbis_comment = tag.vorbis_comments_mut();
        let blank = vec!["<blank>".to_owned()];

        match command.to_lowercase().as_ref() {
            "print" => {
                println!(
                    "{}",
                    format!("\
Title:         {}
Artist:        {}
Album:         {}
Album Artist:  {}
Track:         {}
Total Tracks:  {}
Genre:         {}
Lyrics:        {}",
                        vorbis_comment.title().unwrap_or(&blank)[0],
                        vorbis_comment.artist().unwrap_or(&blank)[0],
                        vorbis_comment.album().unwrap_or(&blank)[0],
                        vorbis_comment.album_artist().unwrap_or(&blank)[0],
                        {
                            let track = vorbis_comment.track();
                            match track {
                               Some(number) => number.to_string(),
                               None => "<blank>".to_owned(), 
                            }
                        },
                        {
                            let total_tracks = vorbis_comment.total_tracks();
                            match total_tracks {
                               Some(number) => number.to_string(),
                               None => "<blank>".to_owned(), 
                            }
                        },
                        vorbis_comment.genre().unwrap_or(&blank)[0],
                        {
                            let lyrics = vorbis_comment.lyrics();
                            match lyrics {
                                Some(_) => "use GET LYRICS command to see",
                                None => "<blank>",
                            }
                        },
                    )
                );            
            },
            "setup" => {
                println!("Note: This command overwrites existing fields, even if left blank");
                println!("To leave a field blank, don't type anything and press enter");

                macro_rules! prompt {
                    ($s:expr, $f:ident) => {
                        print!($s);
                        let mut buffer = String::new();
                        io::stdout().flush().ok().expect("Couldn't flush stdout");
                        io::stdin().read_line(&mut buffer).expect("Couldn't read from stdin");
                        buffer.pop();
                        if !buffer.is_empty() {
                            vorbis_comment.$f(vec![buffer]);
                        }
                    };
                    ($s:expr, $f:ident, number) => {
                        print!($s);
                        let mut buffer = String::new();
                        io::stdout().flush().ok().expect("Couldn't flush stdout");
                        io::stdin().read_line(&mut buffer).expect("Couldn't read from stdin");
                        buffer.pop();
                        if !buffer.is_empty() {
                            vorbis_comment.$f(buffer.parse::<u32>().expect("Not a valid number"));
                        }
                    };
                }

                prompt!("Title: ", set_title);
                prompt!("Artist: ", set_artist);
                prompt!("Album: ", set_album);
                prompt!("Album Artist: ", set_album_artist);
                prompt!("Track: ", set_track, number);
                prompt!("Total Tracks: ", set_total_tracks, number);
                prompt!("Genre: ", set_genre);
                prompt!("Lyrics: ", set_lyrics);
            },
            "get" => {
                assert!(!field.is_empty());

                println!("{}", match field.to_lowercase().as_ref() {
                    "title" => vorbis_comment.title().unwrap_or(&blank)[0].clone(),
                    "artist" => vorbis_comment.artist().unwrap_or(&blank)[0].clone(),
                    "album" => vorbis_comment.album().unwrap_or(&blank)[0].clone(),
                    "albumartist" => vorbis_comment.album_artist().unwrap_or(&blank)[0].clone(),
                    "track" => {
                        let track = vorbis_comment.track();
                        match track {
                            Some(number) => number.to_string(),
                            None => "<blank>".to_owned(),
                        }
                    },
                    "totaltracks" => {
                        let total_tracks = vorbis_comment.total_tracks();
                        match total_tracks {
                            Some(number) => number.to_string(),
                            None => "<blank>".to_owned(),
                        }
                    },
                    "genre" => vorbis_comment.genre().unwrap_or(&blank)[0].clone(),
                    "lyrics" => vorbis_comment.lyrics().unwrap_or(&blank)[0].clone(),
                    _ => panic!("Invalid field"),
                });
            },
            "set" => {
                assert!(!field.is_empty());
                assert!(!value.is_empty());

                match field.to_lowercase().as_ref() {
                    "title" => vorbis_comment.set_title(vec![value]),
                    "artist" => vorbis_comment.set_artist(vec![value]),
                    "album" => vorbis_comment.set_album(vec![value]),
                    "albumartist" => vorbis_comment.set_album_artist(vec![value]),
                    "track" => vorbis_comment.set_track(value.parse::<u32>().expect("Not a valid number")),
                    "totaltracks" => vorbis_comment.set_total_tracks(value.parse::<u32>().expect("Not a valid number")),
                    "genre" => vorbis_comment.set_genre(vec![value]),
                    "lyrics" => vorbis_comment.set_lyrics(vec![value]),
                    _ => panic!("Invalid field"),
                }
            },
            _ => panic!("Invalid command"),
        }
    }

    tag.save().expect("Failed to save FLAC tag");
}