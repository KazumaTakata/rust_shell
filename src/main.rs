use regex::Regex;
use std::env;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

struct Token {
    Type: String,
    Data: String,
}

fn main() {
    let mut user_input = String::new();

    let user = env::var("USER").unwrap();
    let home = env::var("HOME").unwrap();
    let mut cwd = env::current_dir().unwrap();

    let lexgroup_and_regex = vec![
        vec!["BUILTIN", r"export\s+|cd\s+"],
        vec!["WORD", r"[a-zA-Z_]\w*"],
        vec!["EQUAL", "="],
        vec!["VARIABLE", r"\$[a-zA-Z_]\w*"],
        vec!["DOLLSIGN", r"\$"],
        vec!["LPAREN", r"\("],
        vec!["RPAREN", r"\)"],
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
        print!("{}:{}$", user, cwd.display());
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

        //for token in &tokens {
        //println!("[{}:{}]", token.Type, token.Data);
        //}
        if tokens.len() > 0 {
            match &tokens[0].Type[..] {
                "" => {}
                "BUILTIN" => {
                    if &tokens[0].Data[..] == "cd" {
                        let path: &str;
                        if tokens.len() == 1 {
                            path = &home;
                        } else {
                            path = &tokens[1].Data[..];
                        }
                        let root = Path::new(path);
                        env::set_current_dir(&root);

                        cwd = env::current_dir().unwrap();
                    }
                }
                "WORD" => {
                    let mut ex_comm = Command::new(&tokens[0].Data[..]);
                    if tokens.len() > 1 {
                        for arg in &tokens[1..] {
                            ex_comm.arg(&arg.Data[..]);
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
                _ => {}
            }
        }

        user_input.clear();
    }
}
