use sqlite::*;
use structopt::StructOpt;
use cli_table::{format::Justify, print_stdout, Table, WithTitle};
use std::fs::*;
use std::path::*;
use rand::seq::SliceRandom;
use std::io::{stdin, stdout, Write};
use directories::{ProjectDirs};


#[derive(StructOpt)]
#[derive(Debug)]
struct Cli {
    #[structopt(short="G", long)]
    get: bool,
    #[structopt(short="S", long)]
    set: bool,
    #[structopt(short="B", long)]
    remove_all: bool
}

trait DatabaseRequest {
    fn connect() -> Option<Connection> {
        if let Some(proj_dirs) = ProjectDirs::from("dev", "L14",  "passmanager") {
            if !Path::new(proj_dirs.config_dir()).exists() {
                let _ = create_dir(proj_dirs.config_dir());
            }
            match sqlite::open(format!("{}/data.db", proj_dirs.config_dir().to_str().unwrap())) {
                Ok(conn) => {
                    conn.execute("CREATE TABLE IF NOT EXISTS manager (id INTEGER PRIMARY KEY AUTOINCREMENT, website TEXT, username TEXT, password TEXT)").unwrap();
                    Option::from(conn)
                },
                Err(_) => None
            }
        } else {
            None
        }

    }

    fn get_by_id(id: u64) -> Option<Vec<PassManager>>;
    fn get_all() -> Option<Vec<PassManager>>;
    fn get_last_id() -> u64;
    fn set(manager: PassManager) -> PassManager;
}
#[derive(Table, Debug)]
struct PassManager {
    #[table(title = "ID", justify = "Justify::Right")]
    id: u64,
    #[table(title = "Website")]
    website: String,
    #[table(title = "Username")]
    username: String,
    #[table(title = "Password")]
    password: String,
}
impl DatabaseRequest for PassManager {

    fn get_by_id(id: u64) -> Option<Vec<PassManager>> {
        match PassManager::connect() {
            Some(conn) => {
                let req = &*format!("SELECT * FROM manager WHERE id = {}", id);
                let mut statement = conn.prepare(req).unwrap();
                let mut list = Vec::new();
                if let State::Row = &statement.next().unwrap() {
                    list.append(&mut vec![PassManager {
                        id: statement.read::<i64>(0).unwrap() as u64,
                        website: statement.read::<String>(1).unwrap(),
                        username: statement.read::<String>(2).unwrap(),
                        password: statement.read::<String>(3).unwrap()
                    }])
                }
                Option::from(list)
            }
            None => None
        }
    }

    fn get_all() -> Option<Vec<PassManager>> {
        match PassManager::connect() {
            Some(conn) => {
                let req = "SELECT * FROM manager";
                let mut statement = conn.prepare(req).unwrap();
                let mut list = Vec::new();

                while let State::Row = &statement.next().unwrap() {
                    list.append(&mut vec![PassManager {
                        id: statement.read::<i64>(0).unwrap() as u64,
                        website: statement.read::<String>(1).unwrap(),
                        username: statement.read::<String>(2).unwrap(),
                        password: statement.read::<String>(3).unwrap()
                    }])
                }
                Option::from(list)
            }
            None => None
        }
    }

    fn get_last_id() -> u64 {
        let conn = PassManager::connect().unwrap();
        let req = "SELECT * FROM manager ORDER BY id DESC LIMIT 1";
        let mut statement = conn.prepare(req).unwrap();

        if let State::Row = &statement.next().unwrap() {
            statement.read::<i64>(0).unwrap() as u64
        } else {
            0
        }

    }

    fn set(mut manager: PassManager) -> PassManager {
        let conn = PassManager::connect().unwrap();
        let req: &str = &format!("INSERT INTO manager (website, username, password) VALUES ('{}', '{}', '{}')", manager.website, manager.username, manager.password);
        conn.execute(req).unwrap();
        manager.id = PassManager::get_last_id();
        return manager
    }
}

struct Input;
impl Input {

    pub fn new(str: &str) -> String {
        let mut input = String::new();

        print!("{}", str);
        let _ = stdout().flush();
        let _ = stdin().read_line(&mut input).unwrap();

        return input
    }

}

fn main() -> std::io::Result<()> {
    let cli = Cli::from_args();
    if cli.get {
        let list = PassManager::get_all().unwrap();
        print_stdout(list.with_title())
    } else if cli.set {
        let username = Input::new("[~] What's you're username? ");
        let website = Input::new("[~] What is the website who would you sign-in? ");
        let password = random_str(16);
        let manager = PassManager::set(PassManager {
            id: PassManager::get_last_id() + 1,
            website,
            username,
            password
        });

        let list = PassManager::get_by_id(manager.id).unwrap();
        print_stdout(list.with_title())
    } else if cli.remove_all {
        let conn = PassManager::connect().unwrap();
        conn.execute("DROP TABLE manager").unwrap();
        Ok(())
    } else {
        println!("hi :) \n");
        println!("for get help");
        println!("           --help or -h");
        Ok(())
    }
}

fn random_str(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmopqrstuvwxyz123456789".to_string().into_bytes();
    let len = str.len();
    for _ in 0..len {
        str.shuffle(&mut rng)
    }

    let shuffled = String::from_utf8(str).unwrap();

    return format!("{}", &shuffled[0..length])
}
