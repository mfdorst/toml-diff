use super::{TomlChange, TomlDiff};
use std::fs::read;
use toml::Value as TomlValue;

#[test]
fn test_string() {
    let (a, b) = get_toml_values("strings_a", "strings_b");
    let diff = TomlDiff::diff(&a, &b);
    let changes = diff.changes;
    assert_eq!(changes.len(), 4);
    assert!(matches!(
        &changes[0],
        TomlChange::Added(Some(key), TomlValue::String(val))
            if key == "b" && val == "def"
    ));
    assert!(matches!(
        &changes[1],
        TomlChange::Deleted(Some(key), TomlValue::String(val))
            if key == "c" && val == "ghi"
    ));
    assert!(matches!(
        &changes[2],
        TomlChange::Added(Some(key), TomlValue::String(val))
            if key == "e" && val == "mno"
    ));
    assert!(matches!(
        &changes[3],
        TomlChange::Added(Some(key), TomlValue::String(val))
            if key == "f" && val == "pqr"
    ));
}

#[test]
fn test_display_string() {
    let diff = get_diff("strings_a", "strings_b");
    let expected = r#"+ b = "def"
- c = "ghi"
+ e = "mno"
+ f = "pqr"
"#;
    assert_eq!(diff, expected);
}

#[test]
fn test_array() {
    let (a, b) = get_toml_values("arrays_a", "arrays_b");
    let diff = TomlDiff::diff(&a, &b);
    let changes = diff.changes;
    assert_eq!(changes.len(), 4);
    assert!(matches!(
        &changes[0],
        TomlChange::Added(Some(key), TomlValue::Array(val))
            if key == "a"
                && matches!(val[0], TomlValue::Integer(1))
                && matches!(val[1], TomlValue::Integer(2))
                && matches!(val[2], TomlValue::Integer(3))
    ));
    assert!(matches!(
        &changes[1],
        TomlChange::Deleted(Some(key), TomlValue::Array(val))
            if key == "c"
                && matches!(val[0], TomlValue::Integer(3))
                && matches!(val[1], TomlValue::Integer(4))
                && matches!(val[2], TomlValue::Integer(5))
    ));
    assert!(matches!(
        &changes[2],
        TomlChange::Deleted(Some(key), TomlValue::Array(val))
            if key == "e"
                && matches!(val[0], TomlValue::Integer(5))
                && matches!(val[1], TomlValue::Integer(6))
                && matches!(val[2], TomlValue::Integer(7))
    ));
    assert!(matches!(
        &changes[3],
        TomlChange::Deleted(Some(key), TomlValue::Array(val))
            if key == "f"
                && matches!(val[0], TomlValue::Integer(6))
                && matches!(val[1], TomlValue::Integer(7))
                && matches!(val[2], TomlValue::Integer(8))
    ));
}

#[test]
fn test_display_array() {
    let diff = get_diff("arrays_a", "arrays_b");
    let expected = r#"+ a = [1, 2, 3]
- c = [3, 4, 5]
- e = [5, 6, 7]
- f = [6, 7, 8]
"#;
    assert_eq!(diff, expected);
}

#[test]
fn test_table() {
    let (a, b) = get_toml_values("tables_a", "tables_b");
    let diff = TomlDiff::diff(&a, &b);
    let changes = diff.changes;
    assert_eq!(changes.len(), 2);
    assert!(matches!(
        &changes[0],
        TomlChange::Added(Some(key), TomlValue::Table(table))
            if key == "b"
                && matches!(&table["c"], TomlValue::String(val) if val == "ghi")
                && matches!(&table["d"], TomlValue::String(val) if val == "jkl")
    ));
    assert!(matches!(
        &changes[1],
        TomlChange::Deleted(Some(key), TomlValue::Table(table))
            if key == "c"
                && matches!(&table["e"], TomlValue::String(val) if val == "nmo")
                && matches!(&table["f"], TomlValue::String(val) if val == "pqr")
    ));
}

#[test]
fn test_display_table() {
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

#[ignore]
#[test]
fn test_nested_table() {
    let (a, b) = get_toml_values("nested_tables_a", "nested_tables_b");
    let diff = TomlDiff::diff(&a, &b);
    let changes = diff.changes;
    assert_eq!(changes.len(), 2);
    assert!(matches!(
        &changes[0],
        TomlChange::Deleted(Some(key), TomlValue::Table(outer))
            if key == "outer"
                && matches!(
                    &outer["inner_b"],
                    TomlValue::Table(inner_b)
                        if matches!(&inner_b["b"], TomlValue::Integer(2))
        )
    ));
    assert!(matches!(
        &changes[1],
        TomlChange::Deleted(Some(key), TomlValue::Table(outer))
            if key == "outer"
                && matches!(
                    &outer["inner_c"],
                    TomlValue::Table(inner_c)
                        if matches!(&inner_c["c"], TomlValue::Integer(3))
        )
    ));
}

#[ignore]
#[test]
fn test_display_nested_table() {
    let diff = get_diff("nested_tables_a", "nested_tables_b");
    println!("{diff}");
    let expected = r#"- [outer.inner_b]
- b = 2
+ [outer.inner_c]
+ c = 3
"#;
    assert_eq!(diff, expected);
}

fn get_toml_values<'a>(a: &str, b: &str) -> (TomlValue, TomlValue) {
    let a = read(format!("./test_data/{a}.toml")).unwrap();
    let b = read(format!("./test_data/{b}.toml")).unwrap();
    let a = String::from_utf8_lossy(&a);
    let b = String::from_utf8_lossy(&b);
    let a: TomlValue = toml::from_str(&a).unwrap();
    let b: TomlValue = toml::from_str(&b).unwrap();
    (a, b)
}

fn get_diff(a: &str, b: &str) -> String {
    let (a, b) = get_toml_values(a, b);
    let diff = TomlDiff::diff(&a, &b);
    diff.to_string()
}
