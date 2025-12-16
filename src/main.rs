#![allow(dead_code)]
#![allow(unused_imports)]

use std::{io::{Read, Write}, path::PathBuf, process::Command, str::FromStr};

pub mod common;
pub mod api_structs;
pub mod semantic_version;
pub mod app;
mod test;

use common::*;
use api_structs::*;
use semantic_version::*;
use app::*;



fn main() {
    
    let mut app = App::new();
    stop_gradle().unwrap();
    app.update_gradle().unwrap();
    app.update_static_info().unwrap();
    app.fetch_version_info().unwrap();
    
    match app.mc_versions.first().map(|(first, _)| app.mc_versions.last().map(|(last, _)| (first.clone(), last.clone()))).flatten() {
        Some((first, last)) => println!("Found {} Minecraft versions from {} to {}", app.mc_versions.len(), last, first),
        None => println!("No Minecraft versions found.")
    }
    
    loop {
        print!("[ralli] {}> ", app.cwd.file_name().map(|s| s.to_str()).flatten().unwrap_or("?"));
        std::io::stdout().flush().unwrap();
        
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let mut parts = line.split_whitespace();
        if let Some(first) = parts.next() {
            match first.to_lowercase().as_str() {
                "quit" | "exit" => break,
                "help" => {
                    
                }
                "stop" => stop_gradle().unwrap(),
                "clean" => {
                    clean_gradle().unwrap();
                    let _ = app.clean_dependencies().inspect_err(|e| println!("{e}"));
                }
                "test" => if let Some(s) = parts.next() {
                    match s.parse::<SemanticVersion>() {
                        Ok(version) => match app.mc_versions.iter().position(|(v, _)| *v == version) {
                            Some(index) => {
                                let _ = app.test_version(index).inspect_err(|e| println!("{e}"));
                            }
                            None => println!("Minecraft version {} not found.", version)
                        }
                        Err(_) => println!("'{}' isn't a version!", s)
                    }
                }
                "up" => match app.get_current_ranges() {
                    Ok(ranges) => match ranges.last() {
                        Some(last) => match &last.end {
                            Some(end) => match app.mc_versions.iter().position(|(v, _)| v.matches_numbers(end)) {
                                Some(index) => {
                                    let _ = app.test_version(index).inspect_err(|e| println!("{e}"));
                                }
                                None => println!("Minecraft version {} not found.", end)
                            }
                            None => println!("No available Minecraft versions later than {}.", app.mc_versions.first().unwrap().0)
                        }
                        None => println!("No known compatible versions yet. Use 'test <version>' instead.")
                    }
                    Err(e) => println!("Could not get known compatible versions: {e}")
                }
                "down" => match app.get_current_ranges() {
                    Ok(ranges) => match ranges.first() {
                        Some(first) => match &first.start {
                            Some(start) => match app.mc_versions.iter().position(|(v, _)| v.matches_numbers(start)) {
                                Some(index) => if index + 1 >= app.mc_versions.len() {
                                    println!("No available Minecraft versions earlier than {}.", app.mc_versions.last().unwrap().0)
                                } else {
                                    let _ = app.test_version(index + 1).inspect_err(|e| println!("{e}"));
                                }
                                None => println!("Minecraft version {} not found.", start)
                            }
                            None => println!("No available Minecraft versions earlier than {}.", app.mc_versions.last().unwrap().0)
                        }
                        None => println!("No known compatible versions yet. Use 'test <version>' instead.")
                    }
                    Err(e) => println!("Could not get known compatible versions: {e}")
                }
                "deps" => {
                    let _ = app.fetch_dependencies().inspect_err(|e| println!("{e}"));
                }
                "confirm" => {
                    let _ = app.confirm_version().inspect_err(|e| println!("{e}"));
                }
                s => println!("Unrecognized command '{s}'. Use 'help' to see available commands.")
            }
        }
    }
    
}
