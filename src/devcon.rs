use std::io::*;

const INPUT_PROMPT: &str = "$ ";

fn output_prompt() {
    print!("{}", INPUT_PROMPT);
    let _ = stdout().flush();
}

pub fn conmain() {
    output_prompt();
    for line in stdin().lines() {
        if line.is_ok() {
            println!("{}", line.unwrap());
        }
        output_prompt();
    }
}
