use libactionkv::ActionKV;


#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
Usage:
    akv_mem FILE get KEY
    akv_mem FILE delete KEY
    akv_mem FILE insert KEY
    akv_mem FILE update KEY
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    let fname = args.get(2).expect(&USAGE);
    let action: &str = args.get(3).expect(&USAGE).as_ref();
    let key: &str = args.get(4).expect(&USAGE).as_ref();
    let maybe_value = args.get(5);

    let path = std::path::Path::new(&fname);

    let mut store = ActionKV::open(path).expect("unable to open file");
    store.load().expect("unable to load data");

    match action {
        "get" => match store.get(key.as_bytes()).unwrap() {
            None => println!("{:?} not found", key),
            Some(value) => {
                let val = String::from_utf8(value).expect("Found invalid UTF-8");
                println!("{:?}", val);
            },
        },
        "delete" => store.delete(key.as_bytes()).unwrap(),
        "insert" => {
            let value: &str = maybe_value.expect(&USAGE).as_ref();
            store.insert(key.as_bytes(), value.as_bytes()).unwrap();
        },
        "update" => {
            let value: &str = maybe_value.expect(&USAGE).as_ref();
            store.update(key.as_bytes(), value.as_bytes()).unwrap();
        },
        "find" => match store.find(key.as_bytes()).unwrap() {
            None => println!("{:?} not found", key),
            Some(value) => {
                let val = String::from_utf8(value.1).expect("Found invalid UTF-8");
                println!("{:?}", val);
            },
        },
        _ => eprintln!("{}", &USAGE),
    }

}