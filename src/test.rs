use super::TomlDiff;
use std::fs::read;
use toml::Value as TomlValue;

#[test]
fn test_string_changes() {
    let diff = get_diff("strings_a", "strings_b");
    let expected = r#"+ b = "def"
- c = "ghi"
+ e = "mno"
+ f = "pqr"
"#;
    assert_eq!(diff, expected);
}

#[test]
fn test_array_changes() {
    let diff = get_diff("arrays_a", "arrays_b");
    let expected = r#"+ a = [1, 2, 3]
- c = [3, 4, 5]
- e = [5, 6, 7]
- f = [6, 7, 8]
"#;
    assert_eq!(diff, expected);
}

#[test]
fn test_table_changes() {
    let diff = get_diff("tables_a", "tables_b");
    let expected = r#"+ [b]
+ c = "ghi"
+ d = "jkl"
- [c]
- e = "nmo"
- f = "pqr"
"#;
    assert_eq!(diff, expected);
}

#[test]
fn test_nested_table_changes() {
    let diff = get_diff("nested_tables_a", "nested_tables_b");
    let expected = r#"- [outer.inner_b]
- b = 2
+ [outer.inner_c]
+ c = 3
"#;
    assert_eq!(diff, expected);
}

fn get_diff(a: &str, b: &str) -> String {
    let a = read(format!("./test_data/{a}.toml")).unwrap();
    let b = read(format!("./test_data/{b}.toml")).unwrap();
    let a = String::from_utf8_lossy(&a);
    let b = String::from_utf8_lossy(&b);
    let a: TomlValue = toml::from_str(&a).unwrap();
    let b: TomlValue = toml::from_str(&b).unwrap();
    let diff = TomlDiff::diff(&a, &b);
    diff.to_string()
}
