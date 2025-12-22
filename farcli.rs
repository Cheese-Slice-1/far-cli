use std::io::{stdin, stdout, Write};
use std::fmt::Debug;
use std::str::FromStr;
use std::convert::TryFrom;

fn main() {
	// growable stack of vectors (only datatype)
	let mut stack: Vec<Vec<f64>> = vec![];
	// optional, growable buffer
	let mut buffer: Option<Vec<f64>> = None;
	
	println!("FAR (Float ARray) interactive console");
	println!("* Type \"#quit\"/\"#q\" to quit");
	println!("* Type \"#stats\"/\"#s\" to print \
	the contents of the stack and the buffer (no pop)");
	println!("* Type \"#clear\"/\"#cls\"/\"#c\" \
	to clear the screen");
	println!("* Type \"#reset\"/\"#r\" to reset \
	the stack and the buffer");
	println!();
	
	// main loop
	loop {
		// get code w custom prompt;
		// this gets defined here cuz it's
		// immutable and i don't want to waste
		// time reassigning the same variable
		// multiple times
		let line: String = match getinput("|>") {
			Ok(val) => val,
			// if an error w the input ocurs
			// stop the loop and exit w code 1
			Err(msg) => {
				println!("[error]\n{msg}");
				std::process::exit(1);
			}
		};
		
		match &line[..] {
			// quit peaceully
			"#quit" | "#q"  => {
				println!("[quit]");
				break;
			},
			
			// prints stack and buffer
			"#stats" | "#s" => {
				println!("[stats]");
				println!("* Stack: {:?}", stack);
				println!("* Buffer: {:?}\n", buffer);
				continue;
			},
			
			// clear screen
			"#clear" | "#cls" | "#c" => {
				// clear screen control character
				print!("{0}[2J{0}[1;1H", 27 as char);
				continue;
			},
			
			// reset the stack and the buffer
			"#reset" | "#r" => {
				stack = vec![];
				buffer = None;
				println!();
				continue;
			},
			_ => {},
		}
		
		if let Err(msg) = interpret(
			&line,
			&mut stack,
			&mut buffer)
		{
			println!("[error]\n{msg}");
		}
		
		// new line for better reqdability
		// of printed things
		println!();
	}
	
	std::process::exit(0);
}

// HELPER FUNCS

fn getinput(prompt: &str) -> Result<String, String> {
	// the usual readline thingy but with
	// custom error management, nothing
	// too speacial :^
	let mut line = String::new();
	print!("{} ", prompt);
	stdout().flush().unwrap();
	if stdin().read_line(&mut line).is_err() {
		return Err("unable to get input from stdin".to_string());
	}
	
	Ok(line.trim().to_owned())
}

// interpret a line and store necessary values
// in a given optional buffer and a stack of
// vectors
fn interpret(
	line: &str, // reference cuz i won't modify nor own it lol
	stack: &mut Vec<Vec<f64>>, // vec acts like a stack; gets values from buffer
	buffer: &mut Option<Vec<f64>>) // optional, growable array as buffer
	-> Result<(), String>
{
	// NOTE: these two exist cuz i am lazy and i
	// need a way to ckeck if x thing is y thing
	// easily :bbb
	
	// reserved characters
	// (includes categories cuz why not :3)
	let commands = [
		';', ':', '~', '@', // stack/buffer manipulation
		',', '.', '$', // I/O
		'+', '-', '*', '/', '^', '%', // arithmetic
		'=', '<', '>', // filters (clone-pop)
		'_', '#', '|', 'o', // vec manipulation
		'?', '!', '`', // codnitions (if false skip line)
		'(', '[', '\\' // loops
	];
	
	// number characters
	let numchars = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
	
	// split line by whitespaces until the
	// start of a line comment
	let processed: String = line
		.chars()
		.take_while(|&ch| ch != '"')
		.collect();
	
	let splitted: Vec<&str> = processed
		.split_whitespace()
		.collect();
	
	let mut replacebuffer = true;
	
	// unit = space-separated piece from line
	// the part that TRULY interprets everything
	// (all hail for loops ig x3)
	for (_i, &unit) in splitted.iter().enumerate() {
		for (i, character) in unit.chars().enumerate() {
			// check if the unit contains any
			// "illegal" (or reserved) characters
			
			// if a reserved char appears mid-unit
			if commands.contains(&character) && unit.len() > 1 {
				// idk why this gave error but inverting the
				// whole '-' thing w/out changing the
				// i < 1 fixed it ._.
				if  character != '.' && !(character == '-' && i < 1) {
					return Err("reserved commands cannot appear as postfix, prefix or infix: ".to_string() + unit);
				}
			} else if !numchars.contains(&character) && !commands.contains(&character) && character != '.' {
				// if char isn't a number or a reserved
				// char (not mid-unit, as it's been
				// already checked)
				return Err("non-reserved, non-numeric sequences aren't allowed: ".to_string() + unit);
			}
			
			// just a number = oll korrekt
		}
		
		// after checking unit, just interpret it
		match unit {
			// push
			";" => {
				match buffer {
					Some(content) => {
						stack.push(content.to_vec());
						*buffer = None;
					},
					_ => stack.push(vec![]),
				}
			},
			
			// clone
			":" => {
				if let Some(last) = stack.last() {
					stack.push(last.to_vec());
				}
			},
			
			// clear stack
			"~" => stack.clear(),
			
			// pop stack and put it in buffer
			"@" => *buffer = stack.pop(),
			
			// pop and print as numbers
			"." => {
				// default msg is "" (nothing lol)
				let mut result = String::new();
				
				if let Some(content) = buffer {
					// if there's a buffer
					result = prettyformat(content.to_vec());
				} else if let Some(content) = stack.pop() {
					// if no buffer
					result = prettyformat(content);
				}
				
				// if result is None, don't do anything
				if !result.is_empty() {
					println!("{result}");
				}
				
				// delete buffer
				*buffer = None;
			},
			
			// pop and print string
			"$" => {
				// default message will be ""
				let mut result: Result<String, String> = Ok("".to_string());
				
				if let Some(content) = buffer {
					// try converting buffer to String
					result = tryconvert(content.to_vec());
				} else if let Some(content) = stack.pop() {
					// try converting popped vec to String
					result = tryconvert(content);
				}
				
				match result {
					Ok(s) => {
						if !s.is_empty() { println!("{s}"); }
					},
					Err(e) => return Err(e),
				}
				
				// delete buffer
				*buffer = None;
			},
			
			// load input as Vec<f64>
			"," => {
				match getinput("||:") {
					Ok(input) => {
						if replacebuffer {
							*buffer = Some(parseinput(&input));
							replacebuffer = false;
						} else if let Some(content) = buffer {
								content.append(&mut parseinput(&input));
						} else { *buffer = Some(parseinput(&input)); }
					},
					Err(e) => return Err(e),
				}
			},
			
			// if buffer and stack.pop are empty,
			// skip the rest
			// (if true)
			"?" => {
				todo!("implement ? ` (if else) and ! ` (ifnot else) interpreting");

				let Some((_, right)) = processed.split_once("?") else { todo!(); };
				
				let mut ifdepth: usize = 0;
				let mut elsepos: usize = 0;
				
				match buffer {
					// if buffer has a value, check if it's
					// empty and if so skip the rest
					
					Some(content) => {
						if content.is_empty() {
							*buffer = None;
							break;
						} else {
							*buffer = None;
							
							let indexed = right
								.chars()
								.enumerate();

							for (i, c) in indexed {
									if ['?', '!'].contains(&c) {
										ifdepth += 1;
									} else if c == '`' {
										ifdepth -= 1;
									}
									
									if ifdepth == 0 {
										elsepos = i;
										break;
									}
							}
							
							let (iftrue, iffalse) = right.split_at(elsepos);
							println!("{{{iftrue} ～ {iffalse}}}");
							continue;
							
							/*if Err(e) = interpret(
								&right[elsepos..]
								stack,
								buffer)
							{
								
							}
							
							break;*/
						}
					},
					_ => {
						// if there's no buffer, check if stack.pop
						// returns a value, check if it's empty, and
						// if so skip rest
						if let Some(content) = stack.pop() {
							if content.is_empty() { break; }
						} else {
							break;
						}
					},
				}
			},
			
			// the exact same as "?" but inverted
			// (if false)
			"!" => {
				match buffer {
					Some(content) => {
						if !content.is_empty() {
							*buffer = None;
							break;
						} else {
							*buffer = None;
						}
					},
					_ => {
						if let Some(content) = stack.pop() {
							if !content.is_empty() { break; }
						}
					},
				}
			},
			
			// check both buffer and stack.pop or
			// stack.pop twice (no buffer), and
			// do the corresponding vecop then store
			// the result in the buffer
			"+" | "-" | "*" | "/" | "^" | "%" => {
				if let Some(content) = buffer {
					*buffer = vecop(
						Some(content.to_vec()),
						stack.pop(),
						unit
							.chars()
							.next()
							.expect("shouldn't happen"));
				} else {
					*buffer = vecop(
						stack.pop(),
						stack.pop(),
						unit
							.chars()
							.next()
							.expect("shouldn't happen"));
				}
			},
			
			// concatenate
			"_" => {
				match buffer {
					Some(ref mut content) => {
						if let Some(second) = stack.pop() {
							*content = [&second[..], &content[..]].concat();
						}
					},
					_ => {
						if let Some(first) = stack.pop() {
							if let Some(second) = stack.pop() {
								*buffer = Some([&second[..], &first[..]].concat());
							} else {
								*buffer = Some(first);
							}
						}
					},
				}
			},
			
			// replace either stack.pop() with
			// buffer or stack.pop() with
			// stack.pop() partially or completely
			"#" => {
				match buffer {
					// "older" is second, "newer" is content/first
					// newer [1 2 3] older [0 0] -> [1 2 3]
					// newer [1] older [0 0 0] -> [1 0 0]
					Some(ref mut content) => {
						if let Some(second) = stack.pop() {
							if content.len() < second.len() {
								*content = [&content[..], &second[content.len()..]].concat();
							}
						}
					},
					_ => {
						if let Some(first) = stack.pop() {
							if let Some(second) = stack.pop() {
								// if there's two elements in the buffer
								if first.len() < second.len() {
									*buffer = Some([&first[..], &second[first.len()..]].concat());
								}
							}
						}
					},
				}
			},
			
			// separate last value from buffer or
			// stack.pop into buffer and unpop
			// modified stack value (if chosen)
			"|" => {
				if let Some(ref mut content) = buffer {
					if let Some(value) = content.pop() {
						*buffer = Some(vec![value]);
					}
				} else if let Some (mut content) = stack.pop() {
					if let Some(value) = content.pop() {
						*buffer = Some(vec![value]);
					}
				}
			},
			
			// round floats so that they resemble
			// integers
			"o" => {
				*buffer = if let Some(content) = buffer {
					Some(content
						.iter_mut()
						.map(|val| val.round())
						.collect())
				} else {
					stack.pop()
						.map(|content| content
							.into_iter()
							.map(|val| val.round())
							.collect())
				}
			},
			
			// filter last stack element using
			// buffer or filter penultimate stack
			// element using last stack.pop
			"=" => {
				let mut result: Vec<f64> = vec![];
				if let Some(pattern) = buffer {
					if let Some(vector) = stack.pop() {
						result = vector
							.into_iter()
							.filter(|val| pattern.contains(val))
							.collect();
					} else {
						result = pattern.to_vec();
					}
					
					//println!("{buffer:?}");
				} else if let Some(pattern) = stack.pop() {
					if let Some(vector) = stack.pop() {
						result = vector
							.into_iter()
							.filter(|val| pattern.contains(val))
							.collect();

					} else {
						result = pattern.to_vec();
					}
				}
					
				if !result.is_empty() {
					*buffer = Some(result);
				} else {
					*buffer = None;
				}

				//println!("{buffer:?}");
			},
			
			// essentially the same as =, but for a
			// "greater than" filter (last element
			// affect penultimate element rule)
			"<" => {
				let mut result: Vec<f64> = vec![];
				
				if let Some(pattern) = buffer {
					if let Some(vector) = stack.pop() {
						result = vector
							.into_iter()
							.filter(|val| pattern.iter().any(|cmp| val > cmp))
							.collect();
					} else {
						result = pattern.to_vec();
					}
				} else if let Some(pattern) = stack.pop() {
					if let Some(vector) = stack.pop() {
						result = vector
							.into_iter()
							.filter(|val| pattern.iter().any(|cmp| val > cmp))
							.collect();
					} else {
						result = pattern.to_vec();
					}
				}

				if !result.is_empty() {
					*buffer = Some(result);
				} else {
					*buffer = None;
				}
			},
			
			// essentiallu the same as =, but for a
			// "less than" filter (last element
			// affect penultimate element rule)
			">" => {
				let mut result: Vec<f64> = vec![];
				
				if let Some(pattern) = buffer {
					if let Some(vector) = stack.pop() {
						result = vector
							.into_iter()
							.filter(|val| pattern.iter().any(|cmp| val < cmp))
							.collect();
					} else {
						result = pattern.to_vec();
					}
				} else if let Some(pattern) = stack.pop() {
					if let Some(vector) = stack.pop() {
						result = vector
							.into_iter()
							.filter(|val| pattern.iter().any(|cmp| val < cmp))
							.collect();
					} else {
						result = pattern.to_vec();
					}
				}

				if !result.is_empty() {
					*buffer = Some(result);
				} else {
					*buffer = None;
				}
			},
			
			// start loop
			"(" => {
				// no values, just break
				if buffer.is_none() && stack.last().is_none() {
					break;
				}

				// line to repeat is the suffix of
				// the  current line passed split at
				// the "(" instruction
				// line "1 ( ." => loopline "."
				let loopline = match processed
					.split_once('(')
				{
					Some((_, last)) => last.trim(),
					_ => processed.trim(),
				};
				
				//println!("{loopline}");
				
				loop {
					if let Err(msg) = interpret(
						loopline,
						stack,
						buffer)
					{
						if &msg != "break" {
							println!("[error]\n{msg}");
						}
						
						break;
						
					}

					/*if buffer.is_none() && stack.last().is_none() {
						break;
					}*/
					
					//println!("loop repetition");
				}
			},
			
			// break loop
			"\\" => return Err("break".to_owned()),
			
			// load an element or initialize buffer
			_ => {
				if let Ok(value) = f64::from_str(unit) {
					if !replacebuffer {
						match buffer {
							// push parsed value if there's
							// a buffer and isn't to be replaced
							Some(ref mut content) => content.push(value),

							// initialize buffer with value
							// if buffer's None
							_ => { *buffer = Some(vec![value]); }
						}
					} else {
						// replace buffer with a new value
						// if buffer's to be replaced
						*buffer = Some(vec![value]);
						replacebuffer = false;
					}
				}
			},
		}
	}

	//println!("{:?}", &splitted);

	Ok(())
}

fn prettyformat<T: Debug>(vec: Vec<T>) -> String {
	if vec.is_empty() { return String::new(); }

	// vec![x, y, z] -> "{x} {x} {z}"
	let mut ret = String::new();
	for element in vec {
		ret.push_str(&format!("{element:?} "));
	}
	ret.trim().to_owned()
}

fn parseinput(input: &str) -> Vec<f64> {
	let splitted: Vec<_> = input
		.split_inclusive(&[' ', '\t'])
		.collect();
	
	let mut result: Vec<f64> = vec![];
	
	for chunk in &splitted {
		if let Ok(value) = f64::from_str(chunk.trim()) {
			result.push(value);
		} else {
			// if chunk isn't a string repredentation
			// of a number, just take the char code
			for (i, character) in chunk.chars().enumerate() {
				// "\10" -> '1' '0'
				// "\ab" -> '\' 'a' 'b'
				// "\\10" -> '\' 1' '0'
				if character == '\\' && i < 1 && chunk.len() > 1{
					// \1 (i=0) -> continue
					if f64::from_str(&chunk[1..]).is_ok() {
						continue;
					} else if f64::from_str(&chunk[2..]).is_ok() && chunk.chars().nth(1) == Some('\\') {
						// if from the 3rd character it can be parsed
						// AND the 2nd character is '\', continue.
						// \\1 (i=0) -> continue
						// \\1 (i=1) -> no continue
						continue;
					}
				}
				
				// push character code as f64
				// there's no implicit conversion from char,
				// so for precaution, i first made it into
				// an u16 and only then into a f64
				result.push(character as u16 as f64);
			}
		}
	}
	
	// I'M DUMB THIS WAS GIVING AN ERROR CZ
	// I PUT A = AFTER "&chunk in &splitted"
	// WHY DIDN'T I SEE THAT LMAOOO ;v;
	result
}

// try to convert a f64 vec to a String
fn tryconvert(vec: Vec<f64>) -> Result<String, String> {
	let mut result = String::new();
	
	for element in vec {
		if let Ok(code) = u8::try_from(element as i64) {
			result.push(char::from(code));
		} else {
			return Err(format!("invalid UTF-8 character code: {}", element as i64).to_string())
		}
	}
	
	Ok(result)
}

fn vecop(
	first: Option<Vec<f64>>,
	second: Option<Vec<f64>>,
	op: char)
	-> Option<Vec<f64>>
{
	let mut result: Vec<f64> = vec![];
	
	/*let greater = |x: Vec<f64>, y: Vec<f64>| -> (Vec<f64>, Vec<f64>) {
		if x.len() > y.len() { (x, y) }
		else { (y, x) }
	};*/
	
	// check both a and b
	match (&first, &second) {
		(Some(vec1), Some(vec2)) => {
			if vec1.len() == 1 {
				// if the first vec has one element...
				let val = vec1[0];
				
				// map the second vec to that value
				return Some(vec2
					.iter()
					.map(|x| {
						match op {
							'+' => x + val,
							'-' => x - val,
							'*' => x * val,
							'/' => x / val,
							'%' => x % val,
							'^' => x.powf(val),
							_ => { panic!("this should not happen"); }
						}})
					.collect());
			} else if vec2.len() == 1 {
				// if the second vec has one element...
				let val = vec2[0];

				// map the first vec to that value
				return Some(vec1
					.iter()
					.map(|x| {
						match op {
							'+' => x + val,
							'-' => x - val,
							'*' => x * val,
							'/' => x / val,
							'%' => x % val,
							'^' => x.powf(val),
							_ => { panic!("this should not happen"); }
						}})
					.collect());
			}
			
			// get big and small one
			//let (big, small) = greater(first, second);
			for (idx, element) in vec1.iter().enumerate() {
				// if the current index is less than b's length,
				// push the sum of a and b; else push a
				if idx >= vec2.len() {
					result.push(*element);
					continue;
				}
				
				match op {
					'+' => result.push(*element + vec2[idx]),
					'-' => result.push(*element - vec2[idx]),
					'*' => result.push(*element * vec2[idx]),
					'/' => result.push(*element / vec2[idx]),
					'^' => result.push(element.powf(vec2[idx])),
					'%' => result.push(*element % vec2[idx]),
					_ => { panic!("this should not happen"); }
				}
				
				//println!("{:?}", result.last());
			}

			if vec1.len() < vec2.len() {
				for element in vec2[vec1.len()..].iter() {
					result.push(*element);
				}
			}
		},
		(Some(val), None) => { result = val.to_vec(); },
		(None, Some(val)) => { result = val.to_vec(); },
		_ => { return None; },
	}
	
	Some(result)
}

/*
fn greater(a: Option<Vec<f64>>, b: Option<Vec<f64>>)
-> (Vec<f64>, Vec<f64>) {
	match (&a, &b) {
		(Some(x), Some(y)) => {
			if x.len() > y.len() {
				(x.to_vec(), y.to_vec())
			} else {
				(y.to_vec(), x.to_vec())
			}
		},
		(Some(x), None) => (x.to_vec(), vec![]),
		(None, Some(x)) => (vec![], x.to_vec()),
		_ => (vec![], vec![]),
	}
}
*/
