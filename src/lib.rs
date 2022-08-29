use std::borrow::Cow;
use std::fmt;

use toml::Value as TomlValue;

pub struct TomlDiff<'a> {
    changes: Vec<TomlChange<'a>>,
}

pub enum TomlChange<'a> {
    Same,
    Added(Cow<'a, str>, &'a TomlValue),
    Deleted(Cow<'a, str>, &'a TomlValue),
}

impl<'a> TomlDiff<'a> {
    /// Return a list of differences between `a` and `b`
    pub fn new(a: &'a TomlValue, b: &'a TomlValue) -> Self {
        Self {
            changes: diff(Cow::Borrowed(""), a, b),
        }
    }
}

impl<'a> fmt::Display for TomlDiff<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for change in &self.changes {
            match change {
                TomlChange::Same => Ok(()),
                // TODO: Don't clone
                TomlChange::Added(key, val) => format_change(f, '+', key.clone(), val),
                TomlChange::Deleted(key, val) => format_change(f, '-', key.clone(), val),
            }?;
        }
        Ok(())
    }
}

fn format_change<'a>(f: &mut fmt::Formatter, prefix: char, key: Cow<'a, str>, val: &'a TomlValue) -> fmt::Result {
    match val {
        TomlValue::String(val) => write!(f, "{prefix} {key} = \"{val}\"\n"),
        val => {
            // TODO: Don't unwrap
            let serialized = toml::to_string(val).unwrap();
            // TODO: Colorize
            write!(f, "{prefix} {key} = {serialized}\n")
        }
    }
}

// Recursive implementation of `TomlDiff::new`
fn diff<'a>(key: Cow<'a, str>, a: &'a TomlValue, b: &'a TomlValue) -> Vec<TomlChange<'a>> {
    match (a, b) {
        (TomlValue::Table(a_tbl), TomlValue::Table(b_tbl)) => {
            let mut a_vec: Vec<_> = a_tbl.iter().collect();
            let mut b_vec: Vec<_> = b_tbl.iter().collect();
            a_vec.sort_by_key(|e| e.0);
            b_vec.sort_by_key(|e| e.0);
            let mut a_it = a_vec.into_iter().peekable();
            let mut b_it = b_vec.into_iter().peekable();
            let mut changes = vec![];
            loop {
                if let (Some((a_key, a_val)), Some((b_key, b_val))) = (a_it.peek(), b_it.peek()) {
                    if a_key == b_key {
                        changes.extend(diff(
                            Cow::Owned(format!("{key}.{a_key}")),
                            a_val,
                            b_val,
                        ));
                        a_it.next();
                        b_it.next();
                    } else {
                        // Keys are sorted low to high, so the lower order key is missing from the
                        // other table
                        if a_key < b_key {
                            changes.push(TomlChange::Added(Cow::Borrowed(a_key), a_val));
                            a_it.next();
                        } else {
                            changes.push(TomlChange::Deleted(Cow::Borrowed(b_key), b_val));
                            b_it.next();
                        }
                    }
                } else {
                    // One of the iterators has ended.
                    // Anything left in a_it is an addition.
                    changes
                        .extend(a_it.map(|(key, val)| TomlChange::Added(Cow::Borrowed(key), val)));
                    // Anything left in b_it is a deletion.
                    changes.extend(
                        b_it.map(|(key, val)| TomlChange::Deleted(Cow::Borrowed(key), val)),
                    );
                    break;
                }
            }

            changes
        }
        (TomlValue::Array(a_vec), TomlValue::Array(b_vec)) => {
            let mut changes = vec![];
            for (i, (a_elem, b_elem)) in a_vec.into_iter().zip(b_vec).enumerate() {
                changes.extend(diff(Cow::Owned(format!("{key}[{i}]")), a_elem, b_elem));
            }
            changes
        }
        (a, b) => {
            if a == b {
                vec![TomlChange::Same]
            } else {
                // TODO: Figure out how to get rid of this clone
                vec![
                    TomlChange::Added(key.clone(), a),
                    TomlChange::Deleted(key, b),
                ]
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::read;
    use toml::Value as TomlValue;
    use super::TomlDiff;

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

    fn get_diff(a: &str, b: &str) -> String {
        let a = read(format!("./test_data/{a}.toml")).unwrap();
        let b = read(format!("./test_data/{b}.toml")).unwrap();
        let a = String::from_utf8_lossy(&a);
        let b = String::from_utf8_lossy(&b);
        let a: TomlValue = toml::from_str(&a).unwrap();
        let b: TomlValue = toml::from_str(&b).unwrap();
        let diff = TomlDiff::new(&a, &b);
        diff.to_string()
    }
}
