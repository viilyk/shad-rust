#![forbid(unsafe_code)]

fn main() {
    use std::io::BufRead;
    let args = std::env::args().collect::<Vec<String>>();
    let file1 = std::fs::File::open(&args[1]).unwrap();
    let reader1 = std::io::BufReader::new(file1);
    let mut set = std::collections::HashSet::new();
    for line in reader1.lines() {
        set.insert(line.unwrap());
    }
    let file2 = std::fs::File::open(&args[2]).unwrap();
    let reader2 = std::io::BufReader::new(file2);
    for line in reader2.lines() {
        let s = set.take(&line.unwrap());
        if let Some(res) = s {
            println!("{}", res);
        }
    }
}
