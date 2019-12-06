use regex::Regex;
use std::env;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::error::Error;
use std::io::prelude::*;

#[derive(Clone, Debug)]
struct Token {
    Type: String,
    Data: String,
}

struct Lex {
    tokens: Vec<Token>,
    pos: usize,
}

struct Env {
    cwd: std::path::PathBuf,
}

impl Lex {
    fn in_bound(&mut self) -> bool {
        return self.tokens.len() > self.pos;
    }

    fn get_cur_token(&mut self) -> Token {
        return self.tokens[self.pos].clone();
    }

    fn advance_token(&mut self) {
        self.pos += 1;
    }
}
#[derive(Debug)]
struct CommandNode {
    command: String,
    args: Vec<String>,
}
#[derive(Debug)]
struct SeqCommandNode {
    Type: String,
    left: Option<Box<SeqCommandNode>>,
    right: Option<Box<CommandNode>>,
}

//impl SeqCommandNode {
//fn print(&self) {
//if self.Type == "term" {
//let refe = self.right.as_ref();
//refe.unwrap().print();
//} else {
//self.left.as_ref().unwrap().print();
//self.right.as_ref().unwrap().print();
//}
//}
//}

//impl CommandNode {
//fn print(&self) {
//print!("{} ", self.command);
//for arg in &self.args {
//print!("{} ", arg);
//}
//print!("\n");
//io::stdout().flush().unwrap();
//}
/*}*/

fn parse_command(lex: &mut Lex) -> SeqCommandNode {
    let mut seq = SeqCommandNode {
        Type: "".to_string(),
        left: None,
        right: None,
    };
    let mut cur_token = lex.get_cur_token();

    let mut first = true;
    while cur_token.Type == "PIPE" || lex.pos == 0 {
        let parsed = parse_simplecommand(lex);

        if first {
            let mut term_seq = SeqCommandNode {
                Type: "term".to_string(),
                right: Some(Box::new(parsed)),
                left: None,
            };
            seq = term_seq;
            first = false;
        } else {
            let mut new_seq = SeqCommandNode {
                Type: "".to_string(),
                left: None,
                right: None,
            };
            new_seq.left = Some(Box::new(seq));
            seq = new_seq;

            seq.right = Some(Box::new(parsed));
        }
        if lex.in_bound() {
            cur_token = lex.get_cur_token();
            lex.advance_token();
        } else {
            break;
        }
    }

    return seq;
}

fn parse_simplecommand(lex: &mut Lex) -> CommandNode {
    let command = lex.get_cur_token();
    let mut args: Vec<String> = Vec::new();
    lex.advance_token();

    while lex.in_bound() && lex.get_cur_token().Type == "WORD" {
        args.push(lex.tokens[lex.pos].Data.clone());
        lex.pos = lex.pos + 1;
    }

    let command_node = CommandNode {
        command: command.Data.clone(),
        args: args,
    };

    return command_node;
}

fn eval_command(seq: &SeqCommandNode, env: &mut Env) -> Option<String> {
    if seq.Type == "term" {
        let stdout = eval_simple_command(&seq.right.as_ref().unwrap(), env, None);
        return stdout;
    } else {
        let stdout = eval_command(&seq.left.as_ref().unwrap(), env);
        let stdout2 = eval_simple_command(&seq.right.as_ref().unwrap(), env, stdout);
        return stdout2;
    }
}

fn eval_simple_command(
    command: &CommandNode,
    env: &mut Env,
    stdin: Option<String>,
) -> Option<String> {
    match &command.command[..] {
        "cd" => {
            let home = env::var("HOME").unwrap();
            let path: &str;
            if command.args.len() == 0 {
                path = &home;
            } else {
                path = &command.args[0][..];
            }
            let root = Path::new(path);
            env::set_current_dir(&root);

            env.cwd = env::current_dir().unwrap();
            return None;
        }

        _ => {
            if stdin != None {
                let process = match Command::new(&command.command[..])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    Err(why) => panic!("couldn't spawn wc: {}", why.description()),
                    Ok(process) => process,
                };
                match process.stdin.unwrap().write_all(stdin.unwrap().as_bytes()) {
                    Err(why) => panic!("couldn't write to wc stdin: {}", why.description()),
                    Ok(_) => {},
                }

                let mut s = String::new();
                match process.stdout.unwrap().read_to_string(&mut s) {
                    Err(why) => panic!("couldn't read wc stdout: {}", why.description()),
                    Ok(_) => {
                        return Some(s)
                    }
                }
            } else {
                let mut ex_comm = Command::new(&command.command[..]);
                if command.args.len() > 0 {
                    for arg in &command.args {
                        ex_comm.arg(&arg[..]);
                    }
                }

                let result = ex_comm.output();

                match result {
                    Ok(output) => {
                        let output_str = String::from_utf8(output.stdout).unwrap();
                        return Some(output_str);
                    }
                    Err(e) => {
                        return None;
                    }
                }
            }
        }
    }
}

fn main() {
    let mut user_input = String::new();

    let user = env::var("USER").unwrap();
    let home = env::var("HOME").unwrap();
    let mut env = Env {
        cwd: env::current_dir().unwrap(),
    };

    let lexgroup_and_regex = [
        ["BUILTIN", r"export\s+|cd\s+"],
        ["WORD", r"[a-zA-Z_]\w*"],
        ["EQUAL", "="],
        ["VARIABLE", r"\$[a-zA-Z_]\w*"],
        ["DOLLSIGN", r"\$"],
        ["LPAREN", r"\("],
        ["RPAREN", r"\)"],
        ["PIPE", r"\|"],
    ];

    let mut regex_string: String = "".to_owned();
    let index = 0;
    for (i, l_a_r) in lexgroup_and_regex.iter().enumerate() {
        if i == 0 {
            let regex_ele = format!(r"(?P<{}>{})", l_a_r[0], l_a_r[1]);
            regex_string.push_str(&regex_ele[..]);
        } else {
            let regex_ele = format!(r"|(?P<{}>{})", l_a_r[0], l_a_r[1]);
            regex_string.push_str(&regex_ele[..]);
        }
    }

    let re = Regex::new(&regex_string[..]).unwrap();

    loop {
        print!("{}:{}$", user, env.cwd.display());
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut user_input)
            .expect("can not get user input");

        let mut tokens: Vec<Token> = Vec::new();
        for cap in re.captures_iter(&user_input) {
            for lexgroup in &lexgroup_and_regex {
                let lex_cate = lexgroup[0];
                match &cap.name(lex_cate) {
                    None => {}
                    Some(x) => {
                        let token = Token {
                            Type: lex_cate.to_owned(),
                            Data: x.as_str().trim().to_owned(),
                        };
                        tokens.push(token);
                    }
                }
            }
        }
        let mut lex = Lex {
            tokens: tokens,
            pos: 0,
        };

        let parsed = parse_command(&mut lex);
//        println!("{:?}", parsed);
        //        parsed.print();
        let output = eval_command(&parsed, &mut env);
        match output {
            None => {}
            Some(x) => {
                print!("{}", x);
                io::stdout().flush().unwrap();

            }
        }

        user_input.clear();
    }
}
