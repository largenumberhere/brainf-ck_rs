use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io::{Read, Write};
use std::rc::Rc;

fn main() {
    let mut args = std::env::args();
    let mut args = args.skip(1);

    let file_name = args
        .next()
        .expect("Need an argument for the input file");
    let file_in = std::fs::read_to_string(file_name.as_str())
        .expect("Unable to read file");


    let file_out_name = args
        .next()
        .expect("Need an argument for the output file");

    println!("{}", file_out_name);
    let mut file_out =
        Rc::new(
            RefCell::new(
            File::create(file_out_name.as_str())
                .expect("failed to open output file")
            )
        );

    let mut context = Context::new(file_in.as_str());

    transpile(&mut context, file_out.clone());
    drop(file_out);

    std::process::Command::new("rustfmt")
        .arg(file_out_name.as_str())
        .output()
        .unwrap();



    // let tmp = format!("out = {}", file_out_name.as_str());

    std::process::Command::new("rustc")
        .arg(file_out_name.as_str())
        // .arg(tmp.as_str())
        .output()
        .unwrap();

    println!("compiled {}", file_name.as_str());
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

        if !stack.is_empty() {
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

fn transpile(context: &mut Context, mut file_out: Rc<RefCell<File>>) {
    let mut parser = Parser::new();
    file_out.borrow_mut().write_fmt(format_args!("use std::io::Write;\n")).unwrap();
    file_out.borrow_mut().write_fmt(format_args!("fn main() {{")).unwrap();
    file_out.borrow_mut().write_fmt(format_args!(r#"
        let mut memory = [0u8; 30000];
        let mut ptr = 0;
    "#)).unwrap();

    while let Some(instruction) = parser.read_next(context) {
        match instruction {
            Instruction::LSquare => {
                let loop_label = format!("L_{}", context.brace_pairs[&context.program_counter]);
                file_out.borrow_mut()
                    .write_fmt(format_args!(
                        r#"'{loop_label}: loop  {{
                            if memory[ptr] == 0 {{
                                break '{loop_label};
                            }}"#
                    ))
                    .unwrap()
            }
            Instruction::RSquare => {
                file_out.borrow_mut().write_fmt(format_args!("}}"))
                    .unwrap()
            }
            Instruction::RArrow => {
                file_out.borrow_mut().write_fmt(format_args!("ptr = ptr.wrapping_add(1);")).unwrap()
            }
            Instruction::LArrow => {
                file_out.borrow_mut().write_fmt(format_args!("ptr = ptr.wrapping_sub(1);")).unwrap()
            }
            Instruction::Dot => {
                file_out.borrow_mut().write_fmt(format_args!(
                    r#"{{
                        let c = memory[ptr] as char;
                        std::io::stdout().lock().write_fmt(format_args!("{{}}", c)).unwrap();
                    }}"#
                ))
                    .unwrap();
            }
            Instruction::Comma => {
                file_out.borrow_mut().write_fmt(format_args!(r#"
                    {{
                        let buffer = [0u8; 1];
                        std::io::stdin().read_exact(&mut buffer).unwrap();
                        memory[ptr] = buffer[0];
                    }}
                "#)).unwrap();

            }
            Instruction::Plus => {
                file_out.borrow_mut().write_fmt(format_args!(r#"
                    memory[ptr] = memory[ptr].wrapping_add(1);"#
                ))
                    .unwrap();
            }
            Instruction::Minus => {
                file_out.borrow_mut().write_fmt(format_args!(r#"
                    memory[ptr] = memory[ptr].wrapping_sub(1);"#
                ))
                    .unwrap();
            }
        }

        context.program_counter +=1;
    }

    file_out.borrow_mut().write_fmt(format_args!("}}")).unwrap()


}

struct Parser{}

enum Instruction {
    LSquare,
    RSquare,
    RArrow,
    LArrow,
    Dot,
    Comma,
    Plus,
    Minus

}

impl Parser {
    fn new() -> Parser {
        Parser{}
    }

    fn read_next(&mut self, context: &mut Context) -> Option<Instruction>  {
        loop {
            match context.instructions.get(context.program_counter) {
                Some(c) => match *c as char {
                    '[' => {
                        return Some(Instruction::LSquare);
                    },
                    ']' => {
                        return Some(Instruction::RSquare);
                    },
                    '>' => {
                        return Some(Instruction::RArrow);
                    },
                    '<' => {
                        return Some(Instruction::LArrow);
                    },
                    '.' => {
                        return Some(Instruction::Dot);
                    },
                    ',' => {
                        return Some(Instruction::Comma);
                    },
                    '+' => {
                        return Some(Instruction::Plus);
                    },
                    '-' => {
                        return Some(Instruction::Minus);
                    },
                    _ => {
                        // skip any comments
                        context.program_counter += 1;
                    }
                }

                None => {
                    // end of program
                    return None;
                }
            }
        }
    }
}


fn interpret(context: &mut Context) {

    let mut parser = Parser::new();
    while let Some(instruction) = parser.read_next(context) {
        match instruction {
            Instruction::LSquare => {
                let destination = context.brace_pairs[&context.program_counter]+1;
                if context.memory[context.pointer] == 0 {
                    context.program_counter = destination;
                } else {
                    context.program_counter += 1;
                }
            }
            Instruction::RSquare => {
                let destination = context.brace_pairs[&context.program_counter];
                context.program_counter = destination;
            }
            Instruction::RArrow => {
                context.pointer += 1;
                context.program_counter += 1;
            }
            Instruction::LArrow => {
                context.pointer -= 1;
                context.program_counter += 1;
            }
            Instruction::Dot => {
                let byte = context.memory[context.pointer];
                let c = byte as char;
                if !c.is_ascii() {
                    print!("{:?}", byte);
                } else {
                    print!("{}", byte as char);
                }
                std::io::stdout().flush().unwrap();
                context.program_counter += 1;
            }
            Instruction::Comma => {
                let mut byte = [0; 1];
                std::io::stdin().read_exact(&mut byte)
                    .unwrap();
                context.memory[context.pointer] = byte[0];
                context.program_counter += 1;
            }
            Instruction::Plus => {
                context.memory[context.pointer] = context.memory[context.pointer].wrapping_add(1);
                context.program_counter += 1;
            }
            Instruction::Minus => {
                context.memory[context.pointer] = context.memory[context.pointer].wrapping_sub(1);
                context.program_counter += 1;
            }
        }
    }

}