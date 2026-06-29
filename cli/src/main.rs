use apk::reader::ApkReader;

fn main() {
    match ApkReader::read("samples/test.apk"){
        Ok(metadata) => println!("{:#?}", metadata),
        Err(err) => eprintln!("Error: {}", err),
    }
}

