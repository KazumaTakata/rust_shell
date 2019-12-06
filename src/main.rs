use regex::Regex;
use std::env;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

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
    cwd: std::path::PathBuf
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

struct CommandNode {
    command: String,
    args: Vec<String>,
}

struct PipedCommandNode {
    commands: Vec<CommandNode>,
}

impl PipedCommandNode {
    fn print(&self) {
        for command in &self.commands {
            command.print();
        }
    }
}

impl CommandNode {
    fn print(&self) {
        print!("{} ", self.command);
        for arg in &self.args {
            print!("{} ", arg);
        }
        print!("\n");
        io::stdout().flush().unwrap();
    }
}

fn parse_command(lex: &mut Lex) -> PipedCommandNode {
    let mut piped = PipedCommandNode {
        commands: Vec::new(),
    };
    let mut cur_token = lex.get_cur_token();

    while cur_token.Type == "PIPE" || lex.pos == 0 {
        let parsed = parse_simplecommand(lex);
        piped.commands.push(parsed);

        if lex.in_bound() {
            cur_token = lex.get_cur_token();
            lex.advance_token();
        } else {
            break;
        }
    }

    return piped;
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

fn eval_command(piped: &PipedCommandNode, env: &mut Env) {
    for command in &piped.commands {
        eval_simple_command(&command, env);
    }

}

fn eval_simple_command(command: &CommandNode,env: &mut Env) {
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
        }

        _ => {
            let mut ex_comm = Command::new(&command.command[..]);
            if command.args.len() > 1 {
                for arg in &command.args {
                    ex_comm.arg(&arg[..]);
                }
            }

            let result = ex_comm.output();

            match result {
                Ok(output) => {
                    io::stdout().write_all(&output.stdout).unwrap();
                }
                Err(e) => {}
            }
        }
    }
}

fn main() {
    let mut user_input = String::new();

    let user = env::var("USER").unwrap();
    let home = env::var("HOME").unwrap();
    let mut env = Env{cwd: env::current_dir().unwrap()};


    let lexgroup_and_regex = vec![
        vec!["BUILTIN", r"export\s+|cd\s+"],
        vec!["WORD", r"[a-zA-Z_]\w*"],
        vec!["EQUAL", "="],
        vec!["VARIABLE", r"\$[a-zA-Z_]\w*"],
        vec!["DOLLSIGN", r"\$"],
        vec!["LPAREN", r"\("],
        vec!["RPAREN", r"\)"],
        vec!["PIPE", r"\|"],
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
      //  parsed.print();
        eval_command(&parsed, &mut env);

        user_input.clear();
    }
}
