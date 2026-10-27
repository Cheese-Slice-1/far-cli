use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::{stdin, stdout, Write};
use std::str::FromStr;
use ecow::{EcoVec, EcoString, eco_vec, eco_format};
use ecow::string::ToEcoString;

fn main() {
    // growable stack of vectors (only datatype)
    let mut stack: EcoVec<EcoVec<f64>> = eco_vec![];
    // optional, growable buffer
    let mut buffer: Option<EcoVec<f64>> = None;

    println!("FAR (Float ARray) interactive console");
    println!("* Type \"#quit\"/\"#q\" to quit");
    println!(
        "* Type \"#stats\"/\"#s\" to print \
	the contents of the stack and the buffer (no pop)"
    );
    println!(
        "* Type \"#clear\"/\"#cls\"/\"#c\" \
	to clear the screen"
    );
    println!(
        "* Type \"#reset\"/\"#r\" to reset \
	the stack and the buffer"
    );
    println!();

    // main loop
    loop {
        // get code w custom prompt;
        // this gets defined here cuz it's
        // immutable and i don't want to waste
        // time reassigning the same variable
        // multiple times
        let line: EcoString = match getinput("|>") {
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
            "#quit" | "#q" | "\0" => {
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
                stack.clear();
                buffer = None;
                println!();
                continue;
            },
            
            _ => {}
        }

        if let Err(msg) = interpret(&line, &mut stack, &mut buffer) {
            println!("[error]\n{msg}");
        }

        // new line for better reqdability
        // of printed things
        println!();
    }

    std::process::exit(0);
}

// HELPER FUNCS

fn getinput(prompt: &str) -> Result<EcoString, EcoString> {
    // the usual readline thingy but with
    // custom error management, nothing
    // too speacial :^
    let mut line = String::new();
    print!("{} ", prompt);
    stdout().flush().unwrap();
    if stdin().read_line(&mut line).is_err() {
        return Err("unable to get input from stdin".into());
    }

    Ok(line.trim().into())
}

// interpret a line and store necessary values
// in a given optional buffer and a stack of
// vectors
fn interpret(
    line: &str,                // reference cuz i won't modify nor own it lol
    stack: &mut EcoVec<EcoVec<f64>>, // vec acts like a stack; gets values from buffer
    buffer: &mut Option<EcoVec<f64>>,
) -> Result<(), EcoString> {
    // NOTE: these two exist cuz i am lazy and i
    // need a way to ckeck if x thing is y thing
    // easily :bbb

    // reserved characters
    // (includes categories cuz why not :3) ĸŀßðđŋħ
    const COMMANDS: [char; 28] = [
        ';', ':', '~', '@', // stack/buffer manipulation
        ',', '%', '.', '$', // I/O
        '+', '-', '*', '/', '^', '\\', // arithmetic
        '=', '<', '>', // filters (clone-pop)
        '&', '#', '|', 'o', // vec manipulation
        '?', '!', '`', // codnitions (if false skip line)
        '(', '[', ')', ']', // loops
    ];

    // split line by whitespaces until the
    // start of a line comment
    let processed: EcoString = line.chars().take_while(|&ch| ch != '"').collect();

    let splitted: EcoVec<&str> = processed.split_whitespace().collect();

    let mut ignore = false;
    let mut loops = EcoVec::<usize>::new();

    // unit = space-separated piece from line
    // the part that TRULY interprets everything
    // (all hail for loops ig x3)
    let mut cur = 0usize;
    while cur < splitted.len() {
        let unit = splitted[cur];
        
        for (i, character) in unit.chars().enumerate() {
            // check if the unit contains any
            // "illegal" (or reserved) characters

            // if a reserved char appears mid-unit
            if COMMANDS.contains(&character) && unit.len() > 1 {
                // idk why this gave error but inverting the
                // whole '-' thing w/out changing the
                // i < 1 fixed it ._.
                if character != '.' && !(character == '-' && i < 1) {
                    return Err(
                        "reserved commands cannot appear as postfix, prefix or infix: ".to_eco_string() + unit,
                    );
                }
            } else if !&character.is_ascii_digit()
                && !COMMANDS.contains(&character)
                && character != '.'
            {
                // if char isn't a number or a reserved
                // char (not mid-unit, as it's been
                // already checked)
                return Err(
                    "non-reserved, non-numeric sequences aren't allowed: ".to_eco_string() + unit,
                );
            }

            // just a number = oll korrekt
        }
        
        // if we're ignoring instructions, pause at these
        if ignore {
            if ["?", "!", "`"].contains(&unit) {
                ignore = false;
            } else {
                cur += 1;
                continue;
            }
        }

        // after checking unit, just interpret it
        match unit {
            // push
            ";" => match buffer {
                Some(content) => {
                    stack.push(content.clone());
                    *buffer = None;
                }
                _ => stack.push(eco_vec![]),
            },

            // clone
            ":" => {
                if let Some(last) = stack.last() {
                    stack.push(last.clone());
                }
            }

            // clear buffer
            "~" => *buffer = None,

            // pop stack and put it in buffer
            "@" => *buffer = stack.pop(),

            // pop and print as numbers
            "." => {
                // default msg is "" (nothing lol)
                let mut result = EcoString::new();

                if let Some(content) = buffer {
                    // if there's a buffer
                    result = prettyformat(content.clone());
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
            }

            // pop and print string
            "$" => {
                // default message will be ""
                let mut result: Result<EcoString, EcoString> = Ok("".to_eco_string());

                if let Some(content) = buffer {
                    // try converting buffer to EcoString
                    result = tryconvert(content.clone());
                } else if let Some(content) = stack.pop() {
                    // try converting popped vec to EcoString
                    result = tryconvert(content);
                }

                let s = result?;
                if !s.is_empty() {
                    println!("{s}");
                }

                // delete buffer
                *buffer = None;
            }

            // load input as EcoVec<f64>, without spaces
            "," => {
                let input = getinput("||:")?;
                *buffer = Some(parseinput(&input, false));
            },
            
            "%" => {
                //todo!("in the making");
                let input = getinput("||:")?;
                *buffer = Some(parseinput(&input, true));
            },

            // if buffer and stack.last() aren't empty,
            // continue, else skip until ?, !, or `
            // (if true)
            "?" => {
                //todo!("implement ? ` (if else) and ! ` (ifnot else) interpreting");
                
                let emptyvec = EcoVec::<f64>::new();
                
                if (buffer.is_none() || *buffer == Some(emptyvec.clone()))
                    && (stack.last().is_none() || stack.last() == Some(&emptyvec)) {
                        ignore = true;
                    }
            }

            // the exact same as "?" but inverted
            // (if false)
            "!" => {
                let emptyvec = EcoVec::<f64>::new();
                
                if !(buffer.is_none() || *buffer == Some(emptyvec.clone()))
                    && !(stack.last().is_none() || stack.last() == Some(&emptyvec)) {
                        ignore = true;
                    }
            },

            // check both buffer and stack.pop or
            // stack.pop twice (no buffer), and
            // do the corresponding vecop then store
            // the result in the buffer
            "+" | "-" | "*" | "/" | "^" | "\\" => {
                if let Some(content) = buffer {
                    *buffer = vecop(
                        Some(content.clone()),
                        stack.pop(),
                        unit.chars().next().expect("shouldn't happen"),
                    );
                } else {
                    *buffer = vecop(
                        stack.pop(),
                        stack.pop(),
                        unit.chars().next().expect("shouldn't happen"),
                    );
                }
            }

            // concatenate
            "&" => match buffer {
                Some(content) => {
                    if let Some(second) = stack.pop() {
                        *content = [&second[..], &content[..]].concat().into();
                    }
                }
                _ => {
                    if let Some(first) = stack.pop() {
                        if let Some(second) = stack.pop() {
                            *buffer = Some([&second[..], &first[..]].concat().into());
                        } else {
                            *buffer = Some(first);
                        }
                    }
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
                    Some(content) => {
                        if let Some(second) = stack.pop()
                            && content.len() < second.len() {
                                *content = [&content[..], &second[content.len()..]].concat().into();
                            }
                        
                    }
                    _ => {
                        if let Some(first) = stack.pop() 
                            && let Some(second) = stack.pop() {
                                // if there's two elements in the buffer
                                if first.len() < second.len() {
                                    *buffer = Some([&first[..], &second[first.len()..]].concat().into());
                                }
                            }
                        
                    }
                }
            }

            // separate last value from buffer or
            // stack.pop into buffer and unpop
            // modified stack value (if chosen)
            "|" => {
                if let Some(content) = buffer
                    && let Some(value) = content.pop() {
                        *buffer = Some(eco_vec![value]);
                    }
                else if let Some(mut content) = stack.pop()
                    && let Some(value) = content.pop() {
                        *buffer = Some(eco_vec![value]);
                    }
            }

            // round floats so that they resemble
            // integers
            "o" => {
                *buffer = if let Some(content) = buffer {
                    Some(content.iter().map(|val| val.round()).collect::<EcoVec<_>>())
                } else {
                    stack
                        .pop()
                        .map(|content| content.into_iter().map(|val| val.round()).collect())
                }
            }

            // filter last stack element using
            // buffer or filter penultimate stack
            // element using last stack.pop
            "=" => {
                let mut result: EcoVec<f64> = eco_vec![];
                if let Some(pattern) = buffer {
                    if let Some(vector) = stack.pop() {
                        result = vector
                            .into_iter()
                            .filter(|val| pattern.contains(val))
                            .collect();
                    } else {
                        result = pattern.clone();
                    }

                    //println!("{buffer:?}");
                } else if let Some(pattern) = stack.pop() {
                    if let Some(vector) = stack.pop() {
                        result = vector
                            .into_iter()
                            .filter(|val| pattern.contains(val))
                            .collect();
                    } else {
                        result = pattern;
                    }
                }

                if !result.is_empty() {
                    *buffer = Some(result);
                } else {
                    *buffer = None;
                }

                //println!("{buffer:?}");
            }

            // essentially the same as =, but for a
            // "greater than" filter (last element
            // affect penultimate element rule)
            "<" => {
                let mut result: EcoVec<f64> = eco_vec![];

                if let Some(pattern) = buffer {
                    if let Some(vector) = stack.pop() {
                        result = vector
                            .into_iter()
                            .filter(|val| pattern.iter().any(|cmp| val > cmp))
                            .collect();
                    } else {
                        result = pattern.clone();
                    }
                } else if let Some(pattern) = stack.pop() {
                    if let Some(vector) = stack.pop() {
                        result = vector
                            .into_iter()
                            .filter(|val| pattern.iter().any(|cmp| val > cmp))
                            .collect();
                    } else {
                        result = pattern;
                    }
                }

                if !result.is_empty() {
                    *buffer = Some(result);
                } else {
                    *buffer = None;
                }
            }

            // essentiallu the same as =, but for a
            // "less than" filter (last element
            // affect penultimate element rule)
            ">" => {
                let mut result: EcoVec<f64> = eco_vec![];

                if let Some(pattern) = buffer {
                    if let Some(vector) = stack.pop() {
                        result = vector
                            .into_iter()
                            .filter(|val| pattern.iter().any(|cmp| val < cmp))
                            .collect();
                    } else {
                        result = pattern.clone();
                    }
                } else if let Some(pattern) = stack.pop() {
                    if let Some(vector) = stack.pop() {
                        result = vector
                            .into_iter()
                            .filter(|val| pattern.iter().any(|cmp| val < cmp))
                            .collect();
                    } else {
                        result = pattern;
                    }
                }

                if !result.is_empty() {
                    *buffer = Some(result);
                } else {
                    *buffer = None;
                }
            }

            // start a loop. either version is fine,
            // they're for aesthetic purposes
            "(" | "[" => {
                loops.push(cur);
            }

            // loop if not empty
            ")" => {
                let emptyvec = EcoVec::<f64>::new();
                if *buffer != Some(emptyvec.clone())
                    && stack.last() != Some(&emptyvec)
                    && let Some(pos) = loops.pop() {
                        cur = pos
                    }
            },
            
            // loop if empty
            "]" => {
                let emptyvec = EcoVec::<f64>::new();
                if (buffer.is_none() || *buffer == Some(emptyvec.clone()))
                    && (stack.last().is_none() || stack.last() == Some(&emptyvec))
                    && let Some(pos) = loops.pop() {
                        cur = pos
                    }
            },

            // load an element or initialize buffer
            _ => {
                let Ok(value) = f64::from_str(unit) else {
                    return Err(eco_format!("parsing error!! can't parse {unit}"))
                };
                
                match buffer {
                    // push parsed value if there's
                    // a buffer and isn't to be replaced
                    Some(content) => {
                        if cur.checked_sub(1).is_some()
                            && splitted[cur - 1].chars().any(|c| c.is_ascii_digit()) {
                                content.push(value)
                            }
                        else {
                            *buffer = Some(eco_vec![value]);
                        }
                    },
                        
                    // initialize buffer with value
                    // if buffer's None
                    _ => *buffer = Some(eco_vec![value])
                }
            }
        }
        
        cur +=1;
    }

    //println!("{:?}", &splitted);

    Ok(())
}

fn prettyformat<T: Debug>(vec: EcoVec<T>) -> EcoString {
    if vec.is_empty() {
        return EcoString::new();
    }

    // eco_vec![x, y, z] -> "{x}, {y}, {z}"
    let mut ret = EcoString::new();
    let vlen = vec.len();
    for i in 0..vlen {
        if i < vlen - 1 {
            ret.push_str(&eco_format!("{:?}, ", vec[i]));
        } else {
            ret.push_str(&eco_format!("{:?}", vec[i]))
        }
    }
    ret.trim().into()
}

fn parseinput(input: &str, keepspace: bool) -> EcoVec<f64> {
    let splitted: EcoVec<_> = input.split_inclusive(char::is_whitespace).collect();

    let mut result: EcoVec<f64> = eco_vec![];

    for chunk in &splitted {
        if let Ok(value) = f64::from_str(chunk.trim()) {
            result.push(value);
            if keepspace {
                let spaces = chunk[chunk.trim().len()..]
                    .chars()
                    .map(|c| c as u16 as f64)
                    .collect::<EcoVec<f64>>();
                result.extend_from_slice(spaces.as_slice());
            }
        } else {
            // if chunk isn't a string repredentation
            // of a number, just take the char code
            for (i, character) in chunk.trim().chars().enumerate() {
                // "\10" -> '1' '0'
                // "\ab" -> '\' 'a' 'b'
                // "\\10" -> '\' 1' '0'
                if character == '\\' && i < 1 && chunk.len() > 1 {
                    // \1 (i=0) -> continue
                    if f64::from_str(&chunk[1..]).is_ok() {
                        continue;
                    } else if f64::from_str(&chunk[2..]).is_ok()
                        && chunk.chars().nth(1) == Some('\\')
                    {
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
            
            if keepspace {
                let spaces = chunk[chunk.trim().len()..]
                    .chars()
                    .map(|c| c as u16 as f64)
                    .collect::<EcoVec<f64>>();
                result.extend_from_slice(spaces.as_slice());
            }
        }
    }

    // I'M DUMB THIS WAS GIVING AN ERROR CZ
    // I PUT A = AFTER "&chunk in &splitted"
    // WHY DIDN'T I SEE THAT LMAOOO ;v;
    result
}

// try to convert a f64 vec to a EcoString
fn tryconvert(vec: EcoVec<f64>) -> Result<EcoString, EcoString> {
    let mut result = EcoString::new();

    for element in vec {
        if let Ok(code) = u8::try_from(element as i64) {
            result.push(char::from(code));
        } else {
            return Err(eco_format!("invalid UTF-8 character code: {}", element as i64));
        }
    }

    Ok(result)
}

fn vecop(first: Option<EcoVec<f64>>, second: Option<EcoVec<f64>>, op: char) -> Option<EcoVec<f64>> {
    /*let greater = |x: EcoVec<f64>, y: EcoVec<f64>| -> (EcoVec<f64>, EcoVec<f64>) {
        if x.len() > y.len() { (x, y) }
        else { (y, x) }
    };*/

    // check both a and b
    match (&first, &second) {
        (Some(vec1), Some(vec2)) if !vec1.is_empty() && !vec2.is_empty() => {
            if vec1.len() == 1 || vec2.len() == 1 {
                // if the first vec has one element...
                let (bigger, val) = if vec1.len() > vec2.len() {
                    (vec1, vec2[0])
                } else {
                    (vec2, vec1[0])
                };

                // map the second vec to that value
                return Some(
                    bigger.iter()
                        .map(|x| match op {
                            '+' => x + val,
                            '-' => x - val,
                            '*' => x * val,
                            '/' => x / val,
                            '\\' => x % val,
                            '^' => x.powf(val),
                            _ => {
                                panic!("this should not happen");
                            }
                        })
                        .collect(),
                );
            }
            
            // make the operations as normal
            let mut res: EcoVec<f64> = eco_vec![];
            for (i, element) in vec1.iter().enumerate() {
                // if the current index is less than b's length,
                // push the sum of a and b; else push a
                if i >= vec2.len() {
                    res.push(*element);
                    continue;
                }
                
                match op {
                    '+' => res.push(*element + vec2[i]),
                    '-' => res.push(*element - vec2[i]),
                    '*' => res.push(*element * vec2[i]),
                    '/' => res.push(*element / vec2[i]),
                    '^' => res.push(element.powf(vec2[i])),
                    '\\' => res.push(*element % vec2[i]),
                    _ => {
                        panic!("this should not happen");
                    }
                }
                
                //println!("{:?}", result.last());
            }
            
            // if there are values we haven't got to,
            // fill them in
            if res.len() < vec2.len() {
                res.extend_from_slice(&vec2[res.len()..]);
            }
            
            Some(res)
        },
        
        (Some(val), None) | (None, Some(val)) => Some(val.clone()),
        
        _ => None
    }
}

