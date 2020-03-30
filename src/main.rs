extern crate rustyline;
extern crate serde_json;

use std::fs;

use rustyline::error::ReadlineError;
use rustyline::{hint::Hinter, Context, completion::{Completer, Pair}};
use rustyline::Editor;

use rustyline_derive::{Helper, Highlighter, Validator};

use serde_json::Value;

trait Command{
    fn act(&self, args: String, status: &mut Status);

    fn complete(&self, _line: &str, _pos: usize, _ctx: &Context<'_>, _status: &Status
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        return Ok((0, vec![]));
    }


    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>, _status: &Status)
            -> Option<String>{
        return None;
    }
}

#[derive(Helper, Validator, Highlighter)]
struct Status{
    is_running : bool,
    json : Value,
}


pub struct PrintCommand{}

fn json_value_to_str(val: &Value) -> String {
    match &val{
        Value::Null => {
            format!("null")
        }
        Value::Bool(true) => {
            format!("true")
        }
        Value::Bool(false) => {
            format!("false")
        }
        Value::Number(n) => {
            format!("{}", n)
        }
        Value::String(s) => {
            format!("\"{}\"", s)
        }
        Value::Array(arr) => {
            let arr: Vec<String> = arr.iter()
                .map(|a|{json_value_to_str(&a)})
                .collect();
            format!("[{}]", arr.join(", "))
        }
        Value::Object(obj) => {
            let obj: Vec<String> = obj.iter()
                .map(|(k, v)|{
                    format!("{}: {}", k, json_value_to_str(&v))
                }).collect();
            format!("{{{}}}", obj.join(", "))
        }
    }
}

fn resolve_path<'a>(v: &'a Value, path: &str) -> Option<&'a Value>{
    match v{
        Value::Object(map)=>{
            let ind = path.find('.');
            match ind {
                Some(ind) => {
                    // owner.access
                    let (key1, key2) = path.split_at(ind);
                    let (_, key2) = key2.split_at(1);
                    match map.get(key1){
                        Some(sub_json)=>{
                            return resolve_path(sub_json, key2);
                        }
                        None => {
                            println!("{} not found", key1);
                            return None;
                        }
                    }
                }
                None => { // print id
                    let path_ = map.get(path);
                    match path_{
                        Some(path) => Some(path),
                        None => {
                            println!("Key {} not found", path);
                            None
                        }
                    }
                }
            }
        }
        Value::Array(arr)=>{
            let ind = path.find('.');
            match ind {
                Some(ind) => {
                    // owner.access.1.id
                    let (key1, key2) = path.split_at(ind);
                    let (_, key2) = key2.split_at(1);
                    let key = key1.parse::<usize>();
                    match key{
                        Ok(key1)=>{
                            match arr.get(key1){
                                Some(sub_json)=>{
                                    return resolve_path(sub_json, key2);
                                }
                                None => {
                                    println!("index {} not found", key1);
                                    return None;
                                }
                            }
                        }
                        _ => {
                            println!("Expected {} to be an integer", key1);
                            return None;
                        }
                    }
                }
                None => { // print id
                    let path_ = path.parse::<usize>();
                    match path_{
                        Ok(path) => {
                            match arr.get(path){
                                Some(v) => Some(v),
                                None => {
                                    println!("index {} not found", path);
                                    return None;
                                }
                            }
                        }
                        _ => {
                            println!("Expected {} to be an integer", path);
                            return None;
                        }
                    }
                }
            }
        }
        _ => {
            // TODO implement further
            println!("No object or array at evel {}", path);
            return None;
        }
    }
}

impl Command for PrintCommand{
    fn act(&self, args: String, status: &mut Status){
        // path: id
        // path: status.code // TODO

        let (_, path) = args.split_at(6);

        let value = resolve_path(&status.json, path);
        match value{
            Some(v) => {
                println!("{}", json_value_to_str(v));
            }
            None => {
                //println!("Path {} not found", path);
            }
        }
    }

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>, status: &Status
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {

        let (_, mut line) = line.split_at(6);

        let mut pos = pos - 6;

        let mut v = &status.json;

        let mut replace_index = 6;

        // print owner.ac
        let dot_pos = line.rfind('.');

        if dot_pos.is_some(){
            let dot_pos = dot_pos.unwrap();

            pos -= dot_pos + 1;
            replace_index += dot_pos + 1;

            let (path, mut extra) = line.split_at(dot_pos);

            extra = if extra.len() > 1 { extra.split_at(1).1} else { extra };

            line = extra;

            let tmp = resolve_path(v, path);

            if tmp.is_some() {
                v = tmp.unwrap();
            } else {
                return Ok((0, vec![]));
            }
        }

        // println!("{:?}", &line[..pos]);

        match v{
            Value::Object(map) => {
                let m = map.keys().filter_map(|key|{
                    if key.starts_with(&line[..pos]){
                        
                        return Some(Pair{
                            display: key.clone(),
                            replacement: key.clone(),
                        });
                    } else {
                        None
                    }
                }).collect();

                return Ok((replace_index, m));
            }
            Value::Array(arr) => {
                let m = (0 .. arr.len())
                    .map(|ind|{
                        return Pair{
                            display: format!("{}", ind),
                            replacement: format!("{}", ind,)
                        };
                    }).collect();
                return Ok((replace_index, m));
            }
            _ => {
                return Ok((replace_index, vec![]))
            }
        }
    }


    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>, status: &Status)
            -> Option<String>{

        let (_, line) = line.split_at(6);

        let pos = pos - 6;

        if pos < line.len() {
            return None;
        }

        match &status.json{
            Value::Object(map) => {
                return map.keys().filter_map(|key|{
                    if key.starts_with(&line[..pos]){
                        return Some(key.clone()[pos..].to_owned());
                    } else {
                        None
                    }
                }).next();
            }
            _ => {
                return None;
            }
        }
    }
}


pub struct HelpCommand{}

impl Command for HelpCommand{
    fn act(&self, _s: String, _status: &mut Status){
        println!("here is your help");
    }
}

pub struct ExitCommand{}

impl Command for ExitCommand{
    fn act(&self, _s: String, status: &mut Status){
        status.is_running = false;
    }
}

pub struct LoadCommand{}

impl Command for LoadCommand{
    fn act(&self, args: String, status: &mut Status){

        let (_,addr) = args.split_at(5);

        println!("Loading {}", addr);

        let file = fs::read_to_string(addr);

        match file{
            Ok(content) => {
                let json = serde_json::from_str(content.as_str());

                match json{
                    Ok(content) => {
                        status.json = content;
                    }
                    Err(err) => {
                        println!("Unable to parse file");
                        println!("Error: {}", err);
                    }
                }
                
            }
            Err(err) => {
                println!("Unable to open file {}", addr);
                println!("Error: {}", err);
            }
        }
    }
}

fn get_command(s: &str) -> Option<Box<dyn Command>>{
    match s{
        "help" => {
            Some(Box::new(HelpCommand{}))
        }
        "load" => {
            Some(Box::new(LoadCommand{}))
        }
        "print" => {
            Some(Box::new(PrintCommand{}))
        }
        "exit" => {
            Some(Box::new(ExitCommand{}))
        }
        _ => {
            None
        }
    }
}

fn get_commands() -> Vec<String> {
    return vec![
        "help".to_string(),
        "load".to_string(),
        "print".to_string(),
        "exit".to_string(),
    ];
}

impl Completer for Status{
        type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {

        if line.find(' ').is_some(){
            // TODO ask command for completion
            let split:Vec<_> = line.split(' ').collect();
            let command = split[0];

            let command = get_command(command);

            match command {
                Some(b) => {
                    return (*b).complete(line, pos, ctx, self);
                }
                None => {
                    return Ok((0, vec![]));
                }
            }

        } else {
            let m = get_commands()
                .iter()
                .filter_map(|compl|{
                    if compl.starts_with(&line[..pos]){
                        let compl = compl.to_owned().to_string();

                        return Some(Pair{
                            display: compl.clone(),
                            replacement: compl,
                        });

                    } else {
                        None
                    }
                }).collect();

            return Ok((0, m));
        }
    }
}

impl Hinter for Status{
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String>{
        if pos < line.len() {
            return None;
        }

        if line.find(' ').is_some(){
            // TODO ask command for completion
            let split:Vec<_> = line.split(' ').collect();
            let command = split[0];

            let command = get_command(command);

            match command {
                Some(b) => {
                    return (*b).hint(line, pos, ctx, self);
                }
                None => {
                    None
                }
            }

        } else {


            get_commands()
                .iter()
                .filter_map(|hint| {
                    // expect hint after word complete, like redis cli, add condition:
                    // line.ends_with(" ")
                    if pos > 0 && hint.starts_with(&line[..pos]) {
                        Some(hint[pos..].to_owned())
                    } else {
                        None
                    }
                })
                .next()
        }
    }
}

fn act(s: String, status: &mut Status){
    // command argument0 argument1 ...
    let split:Vec<_> = s.split(' ').collect();
    let command = split[0];


    // TODO implement act
    let command = get_command(command);

    match command {
        Some(b) => {
            (*b).act(s, status);
        }
        None => {
            
        }
    }
    
    //return status;
}

fn main() {
    // `()` can be used when no completer is required
    let status = Status{
        is_running: true,
        json: Value::Null,
    };


    let mut rl = Editor::<Status>::new();

    rl.set_helper(Some(status));


    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }


    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                let status = rl.helper_mut().unwrap();

                act(line.as_str().to_string(), status);

                if !status.is_running{
                    break;
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }

    }
    rl.save_history("history.txt").unwrap();
}
