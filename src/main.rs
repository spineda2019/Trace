/*
 *  main.rs - CLI setup and pass along to profiler
 *  Copyright (C) 2024  Sebastian Pineda (spineda.wpi.alum@gmail.com)
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along
 *  with this program. If not, see <https://www.gnu.org/licenses/>
 */

mod logger;

use std::path::PathBuf;

use argparse::{ArgumentParser, Store, StoreTrue};
use logger::Logger;

const VERSION: &str = "0.0.1";

fn print_version_info() {
    println!("Trace version {}", VERSION);
}

fn main() -> Result<(), std::io::Error> {
    let mut logging: bool = false;
    let mut print_version: bool = false;
    let mut directory: String = String::new();

    {
        let mut argument_parser: ArgumentParser = ArgumentParser::new();
        argument_parser.set_description("Check your codebase FAST");

        argument_parser.refer(&mut logging).add_option(
            &["-l", "--log"],
            StoreTrue,
            "Output logging of file locations",
        );

        argument_parser.refer(&mut print_version).add_option(
            &["-v", "--version"],
            StoreTrue,
            "Print version and license information",
        );

        argument_parser.refer(&mut directory).add_option(
            &["-d", "--directory"],
            Store,
            "Directory you would like to profile",
        );

        argument_parser.parse_args_or_exit();
    }

    if print_version {
        print_version_info();
        return Ok(());
    }

    let current_dir: PathBuf = std::env::current_dir()?;

    let logger = Logger::new(&directory, &current_dir, logging);
    logger.log()?;

    Ok(())
}
