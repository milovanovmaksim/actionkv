use std::collections::HashMap;

use libactionkv::ActionKV;

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
Usage:
    akv_mem FILE get KEY
    akv_mem FILE delete KEY
    akv_mem FILE insert KEY VALUE
    akv_mem FILE update KEY VALUE
";


type ByteStr = [u8];
type ByteString = Vec<u8>;


fn store_index_on_disk(a: &mut ActionKV, index_key: &ByteStr) {
    a.index.remove(index_key);
    let index_as_bytes = bincode::serialize(&a.index).unwrap();
    a.index = std::collections::HashMap::new();
    a.insert(index_key, &index_as_bytes).unwrap();
}

fn main() {
    const INDEX_KEY: &ByteStr = b"+index";
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    let fname = args.get(1).expect(&USAGE);
    let action: &str = args.get(2).expect(&USAGE).as_ref();
    let key: &str = args.get(3).expect(&USAGE).as_ref();
    let maybe_value = args.get(4);

    let path = std::path::Path::new(&fname);

    let mut store = ActionKV::open(path).expect("unable to open file");
    store.load().expect("unable to load data");

    match action {
        "get" => {
            let index_as_bytes = store.get(&INDEX_KEY).unwrap().unwrap();
            let index_decoded = bincode::deserialize(&index_as_bytes);
            let index: HashMap<ByteString, u64> = index_decoded.unwrap();
            match index.get(key.as_bytes()) {
                None => eprintln!("{:?} not found", key),
                Some(&i) => {
                    let kv = store.get_at(i).unwrap();
                    let val = String::from_utf8(kv.value).expect("Found invalid UTF-8");
                    println!("{:?}", val);
                }
            }
        }
        "delete" => {
            store.delete(key.as_bytes()).unwrap();
            store_index_on_disk(&mut store, INDEX_KEY);
         },
        "insert" => {
            let value: &str = maybe_value.expect(&USAGE).as_ref();
            store.insert(key.as_bytes(), value.as_bytes()).unwrap();
            store_index_on_disk(&mut store, INDEX_KEY);
        }
        "update" => {
            let value: &str = maybe_value.expect(&USAGE).as_ref();
            store.update(key.as_bytes(), value.as_bytes()).unwrap();
            store_index_on_disk(&mut store, INDEX_KEY);
        }
        "find" => match store.find(key.as_bytes()).unwrap() {
            None => println!("{:?} not found", key),
            Some(value) => {
                let val = String::from_utf8(value.1).expect("Found invalid UTF-8");
                println!("{:?}", val);
            }
        },
        _ => eprintln!("{}", &USAGE),
    }
}
