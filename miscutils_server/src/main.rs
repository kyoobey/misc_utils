


use clappers::Clappers;                            // commandline arguments parser
use fancy::{ printcol, printcoln, eprintcoln };    // ansi (colored output) text in terminal
use rand::{ Rng, thread_rng };                     // random numbers

#[macro_use]
extern crate log;                                  // logger
use simple_logger;

use std::{
	io::{ BufReader, prelude::* },                 // for writing responses to eastablished tcp connections
	net::{ TcpListener, TcpStream },               // make a tcp listener for the server
	process::exit                                  // instantly exit the program
};

use hexpng::generate_png;                          // generate png from hex code

use miscutils_server::ThreadPool;



fn handle_connection (mut stream: TcpStream) {

	let buffer_reader = BufReader::new(&mut stream);

	let request_line = {
		buffer_reader
			.lines()
			.next().unwrap_or_else(|| {
				warn!("empty request ¯\\_(ツ)_/¯");
				Ok("".to_string())
			})
			.unwrap_or_else(|e| {
				warn!("{:?}", e);
				warn!("error while reading tcp request");
				"".to_string()
			})
	};



	let mut status = "HTTP/1.0 400 NOT FOUND";
	let mut bytes: Vec<u8> = include_bytes!("html/404.html").to_vec();

	if request_line.contains("GET / HTTP") {
		status = "HTTP/1.0 200 OK";
		bytes = include_bytes!("html/index.html").to_vec();
	} else if request_line.contains("GET /hexpng/") {
		let col = {
			let s = &request_line[12..request_line.len()-9];
			if s.len()%2==0 {
				(0..s.len())
					.step_by(2)
					.map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
					.collect::<Vec<u8>>()
			} else { vec![] }
		};

		if col.len() == 3 {
			status = "HTTP/1.0 200 OK";
			bytes = generate_png(col[0], col[1], col[2], 255);
		} else if col.len() == 4 {
			status = "HTTP/1.0 200 OK";
			bytes = generate_png(col[0], col[1], col[2], col[3]);
		}
	}



	let length = bytes.len();
	let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n");
	let response = response.bytes();
	let response = response.chain(bytes.into_iter()).collect::<Vec<u8>>();

	stream.write_all(&response[..]).unwrap_or_else(|e| {
		warn!("{:?}", e);
		warn!("error while writing tcp response");
		error!("exiting (probably can recover)\n");
	});

}



fn main () {



	// parse command line arguments
	let commandline_arguments = Clappers::build()
									.set_flags(vec![
										"h|help",
										"q|quite"
									])
									.set_singles(vec![
										"p|port",
										"t|threads",
										"l|loglevel"
									])
									.parse();

	if commandline_arguments.get_flag("help") {
		println!("
a server which is a toy collection of miscellaneous utilities

usage: miscutils_server [arguments]

arguments:
	-p --port     [n]    port to connect the server 1-{0}
	-t --threads  [n]    number of threads/workers in the threadpool 1-{0}
	-l --loglevel [n]    set the log level 0-3
	-h --help            print this help text
", u16::MAX);

		exit(0);
	}



	// ----------------------------------------------- print banner

	if !commandline_arguments.get_flag("quite") {

		// warning stating that ansi is required
		print!("if the text appears wierd, you're probably using windows
cmd/cmdhost, just enable ANSI by running the following
command in a cmd:
\tREG ADD HKCU\\CONSOLE /f /v VirtualTerminalLevel /t REG_DWORD /d 1\n\n");

		// remove the previous warning using ansi in supported terminals
		// other languages can accept \033 (octal) but rust only accepts \x1b
		// \x1b[2K -> clear current line
		// \x1b[A  -> move cursor up
		print!("\r\x1b[A\x1b[A\x1b[2K\x1b[A\x1b[A\x1b[A\x1b[2K");


		printcol!("
[bold|red]========================================================[:]
        [bold|magenta]Welcome [def]to my ");

		let mut rng = thread_rng();
		match rng.gen_range(0..5) {
			0 => { printcol!("[bold|black]shitty"); },
			1 => { printcol!("[bold|green]shitty"); },
			2 => { printcol!("[bold|yellow]shitty"); },
			3 => { printcol!("[bold|blue]shitty"); },
			4 => { printcol!("[bold|magenta]shitty"); },
			_ => unreachable!()
		}
		printcoln!("[bold] miscutils webserver
[red]========================================================\n");

	}

	// ------------------------------------------------------------



	// setup logger
	println!("initializing logger...");

	simple_logger::SimpleLogger::new()
		.with_level(log::LevelFilter::Debug)
		.init()
		.unwrap_or_else(|e| {
			eprintcoln!("[bold|red] {:?}\ncouldn't initialize logger :(, there's something seriously wrong\n", e);
			exit(1);
		});

	println!("logger initialized, any further output will be through the logger\n");



	// ----------------------------------------- setup tcp listener

	let port: u16 = commandline_arguments.get_single("port").parse().unwrap_or_else(|e| {
		warn!("error parsing port: {:?}", e);
		warn!("will use port 6969");
		6969
	});

	info!("creating tcp listener and binding to 0.0.0.0:{port}");

	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap_or_else(|e| {
		error!("{:?}", e);
		error!("couldn't bind to 0.0.0.0:{} ¯\\_(ツ)_/¯ ... exiting\n", port);
		exit(1);
	});

	info!("bound to 0.0.0.0:{}{}", port, if port == 6969 { ", nice" } else { "" });

	// ------------------------------------------------------------



	// setup threadpool
	let threads: u16 = commandline_arguments.get_single("threads").parse().unwrap_or_else(|e| {
		warn!("error parsing threads: {:?}", e);
		warn!("will use threads=4");
		4
	});

	info!("creating thread pool");

	let pool = ThreadPool::new(threads.into());



	// --------------------- accept and handle incoming connections

	for stream in listener.incoming().take(16) {
		let stream = stream.unwrap();

		// debug!("connection established");

		pool.execute(|| {
			handle_connection(stream);
		});
	}

	// ------------------------------------------------------------



	// exit
	warn!("exiting server gracefully :)");

}


