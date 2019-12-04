use std::io;
use std::env;
use std::path::Path;
use std::io::Write;
use std::process::Command;




fn main() {
    println!("Hello, world!");
    let mut user_input = String::new();
    
    let user = env::var("USER").unwrap();
    let home = env::var("HOME").unwrap();
    let mut cwd = env::current_dir().unwrap();
    loop {
        print!("{}:{}$", user, cwd.display());
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut user_input) 
            .expect("can not get user input");
        let command_and_args :Vec<&str> = user_input.trim().split_whitespace().collect();
        
        let command;

        if command_and_args.len() > 0 {
            command = command_and_args[0];
        } else {
            command = "";
        }
        
        match command {
            "" => {},
            "cd" => {
                let path:&str;
                if command_and_args.len() == 1 {
                    path = &home;
                } else {
                    path = command_and_args[1];
                } 
                let root = Path::new(path);
                env::set_current_dir(&root);                

                cwd = env::current_dir().unwrap();
 

            },
            _ => { 
            let mut ex_comm = Command::new(command);
            if command_and_args.len() > 1 {
                for arg in &command_and_args[1..] {
                    ex_comm.arg(arg);
                }
            } 

            let result = ex_comm.output();

            match result {
                Ok(output) => {
                    io::stdout().write_all(&output.stdout).unwrap();
                },
                Err(e) => {

                },
            }
            },
        }
         
       
        user_input.clear();

        //for his in &tree.root.child {
            //print!("{}", his.ch);
                //io::stdout().flush().unwrap();
        //}
        //println!("");        

    }
}
