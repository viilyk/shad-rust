#![forbid(unsafe_code)]

use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////

pub type IniFile = HashMap<String, HashMap<String, String>>;

pub fn parse(content: &str) -> IniFile {
    let lines = content.lines();
    let mut current_section = String::new();
    let mut ini = HashMap::new();
    for mut s in lines {
        s = s.trim();
        if s.starts_with('[') {
            current_section = parse_section(s).to_string();
            if !ini.contains_key(&current_section) {
                ini.insert(current_section.to_string(), HashMap::new());
            }
        } else if !s.is_empty() {
            let (key, value) = parse_pair(s);
            let map = ini.get_mut(&current_section).unwrap();
            map.insert(key.to_string(), value.to_string());
        }
    }
    ini
}

fn parse_section(line: &str) -> &str {
    let Some(x) = line.strip_prefix('[') else {
        panic!("AAA")
    };
    if let Some(y) = x.strip_suffix(']') {
        if y.contains('[') || y.contains(']') {
            panic!("Incorrect section: {}", line);
        }
        y
    } else {
        panic!("Incorrect section: {}", line);
    }
}

fn parse_pair(line: &str) -> (&str, &str) {
    let mut iter = line.split('=');
    let key = iter.next().unwrap().trim();
    let value = iter.next();
    if iter.next().is_some() {
        panic!("AAAA");
    }
    if let Some(v) = value {
        (key, v.trim())
    } else {
        (key, "")
    }
}
