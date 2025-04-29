use std::collections::HashMap;
use std::io::{Read, Write};

fn main() {
    let file_name = std::env::args()
        .nth(1)
        .expect("Need an argument for the file to run");

    let code = std::fs::read_to_string(file_name)
        .expect("Unable to read file");

    let mut context = Context::new(&code);
    interpret(&mut context);

}

#[derive(Debug)]
struct Context{
    instructions: Vec<u8>,
    pointer: usize,
    memory: [u8; 30000],
    program_counter: usize,
    brace_pairs: HashMap<usize, usize>,
}

impl Context {
    fn new(code: &str) -> Context {
        let instructions = code.as_bytes().to_vec();

        let mut stack = Vec::default();
        let mut brace_pairs = HashMap::new();

        for (i, c) in instructions.iter().enumerate() {
            let c = *c as char;
            if c == '[' {
                stack.push(i);
            } else if c == ']' {
                let other = stack.pop()
                    .expect("unmatched closing square bracket");

                brace_pairs.insert(i, other);
                brace_pairs.insert(other, i);
            }
        }

        if stack.is_empty() {
            panic!("Unmatched opening square bracket");
        }

        let memory = [0u8; 30000];

        Context {
            instructions,
            brace_pairs,
            program_counter: 0,
            memory,
            pointer: 0,
        }
    }
}

fn interpret(context: &mut Context) {
    loop {
        match context.instructions.get(context.program_counter) {
            Some(c) => match *c as char {
                '[' => {
                    let destination = context.brace_pairs[&context.program_counter]+1;
                    if context.memory[context.pointer] == 0 {
                        context.program_counter = destination;
                    } else {
                        context.program_counter += 1;
                    }
                },
                ']' => {
                    let destination = context.brace_pairs[&context.program_counter];
                    context.program_counter = destination;
                },
                '>' => {
                    context.pointer += 1;
                    context.program_counter += 1;
                },
                '<' => {
                    context.pointer -= 1;
                    context.program_counter += 1;
                },
                '.' => {
                    let byte = context.memory[context.pointer];
                    let c = byte as char;
                    if !c.is_ascii() {
                        print!("{:?}", byte);
                    } else {
                        print!("{}", byte as char);
                    }
                    std::io::stdout().flush().unwrap();
                    context.program_counter += 1;
                },
                ',' => {
                    let mut byte = [0; 1];
                    std::io::stdin().read_exact(&mut byte)
                        .unwrap();
                    context.memory[context.pointer] = byte[0];
                    context.program_counter += 1;
                },
                '+' => {
                    context.memory[context.pointer] = context.memory[context.pointer].wrapping_add(1);
                    context.program_counter += 1;
                },
                '-' => {
                    context.memory[context.pointer] = context.memory[context.pointer].wrapping_sub(1);
                    context.program_counter += 1;
                },
                _ => {
                    // ignore other characters
                    context.program_counter +=1;
                }
            }

            None => {
                // end of program
                return;
            }
        }
    }

}