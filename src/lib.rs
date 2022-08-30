use std::borrow::Cow;
use std::fmt;
use std::mem::discriminant;

use toml::Value as TomlValue;

pub struct TomlDiff<'a> {
    changes: Vec<TomlChange<'a>>,
}

pub enum TomlChange<'a> {
    Same,
    Added(Cow<'a, str>, &'a TomlValue),
    Deleted(Cow<'a, str>, &'a TomlValue),
    Changed(Option<Cow<'a, str>>, &'a TomlValue, &'a TomlValue),
}

impl<'a> fmt::Display for TomlDiff<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for change in &self.changes {
            match change {
                TomlChange::Same => Ok(()),
                // TODO: Don't clone
                TomlChange::Added(key, val) => format_change(f, '+', key.clone(), val),
                TomlChange::Deleted(key, val) => format_change(f, '-', key.clone(), val),
                TomlChange::Changed(key, val_a, val_b) => {
                    todo!()
                }
            }?;
        }
        Ok(())
    }
}

fn format_change<'a>(
    f: &mut fmt::Formatter,
    prefix: char,
    key: Cow<'a, str>,
    val: &'a TomlValue,
) -> fmt::Result {
    match val {
        TomlValue::String(val) => write!(f, "{prefix} {key} = \"{val}\"\n"),
        TomlValue::Table(table) => {
            write!(f, "{prefix} [{key}]\n")?;
            for (key, val) in table {
                format_change(f, prefix, Cow::Borrowed(key), val)?;
            }
            Ok(())
        }
        val => {
            // TODO: Don't unwrap
            let serialized = toml::to_string(val).unwrap();
            // TODO: Colorize
            write!(f, "{prefix} {key} = {serialized}\n")
        }
    }
}

impl<'a> TomlDiff<'a> {
    /// Return a list of differences between `a` and `b`. A is considered "new" and `b` is
    /// considered "old", so items missing from `a` are considdered "deletions", while items
    /// missing from `b` are considered "additions".
    ///
    /// Changes in table keys are always considdered either "deletions" or "additions", while
    /// changes in the value of a key are considdered "changes".
    pub fn diff(a: &'a TomlValue, b: &'a TomlValue) -> Self {
        match (a, b) {
            (TomlValue::Table(_), TomlValue::Table(_)) => {}
            _ => panic!("Expected a table at the top level"),
        }
        let mut changes = vec![];
        let mut stack = vec![(a, b)];
        while let Some((a, b)) = stack.pop() {
            if a.is_array() {
                // We only ever push pairs of the same type to `stack`
                let a_vec = a.as_array().unwrap();
                let b_vec = b.as_array().unwrap();
                let mut a_it = a_vec.into_iter();
                let mut b_it = b_vec.into_iter();

                // TODO: Ideally we would sort elements first, then track additions and
                // deletions as we do for keys in Tables, but TomlValue does not implement Ord,
                // so we can't sort. We could get around this by implementing Ord for
                // TomlValue.
                for (a_elem, b_elem) in a_it.by_ref().zip(b_it.by_ref()) {
                    if a_elem == b_elem {
                        // No change in this array element
                        continue;
                    }
                    if discriminant(a_elem) != discriminant(b_elem) {
                        // Elements have different types
                        changes.push(TomlChange::Changed(None, a_elem, b_elem));
                        continue;
                    }
                    if a_elem.is_table() || a_elem.is_array() {
                        stack.push((a_elem, b_elem));
                    } else {
                        changes.push(TomlChange::Changed(None, a_elem, b_elem));
                    }
                }
                todo!("Process the leftovers if the arrays have different lengths")
            }
            // We only ever push `Array`s or `Table`s to `stack`
            let a_map = a.as_table().unwrap();
            let b_map = b.as_table().unwrap();
            let mut a_elems: Vec<_> = a_map.iter().collect();
            let mut b_elems: Vec<_> = b_map.iter().collect();
            a_elems.sort_by_key(|e| e.0);
            b_elems.sort_by_key(|e| e.0);
            let mut a_elems_it = a_elems.into_iter().peekable();
            let mut b_elems_it = b_elems.into_iter().peekable();

            while let (Some((&ref a_key, &ref a_val)), Some((&ref b_key, &ref b_val))) =
                (a_elems_it.peek(), b_elems_it.peek())
            {
                // Keys are sorted low to high, so if the keys are different, that means
                // that the lesser key is missing from the other table.
                if a_key < b_key {
                    // Keys missing from `b` are considdered "added" in `a`
                    changes.push(TomlChange::Added(Cow::Borrowed(a_key), a_val));
                    a_elems_it.next();
                    continue;
                } else if a_key > b_key {
                    // Keys missing from `a` are considered "deleted" from `b`
                    changes.push(TomlChange::Deleted(Cow::Borrowed(b_key), b_val));
                    b_elems_it.next();
                    continue;
                }
                // Keys are the same
                if a_val == b_val {
                    // No change in this key-value pair
                    continue;
                }
                // Keys are the same, but the value is different
                if discriminant(a_val) != discriminant(b_val) {
                    // Values have different types
                    changes.push(TomlChange::Changed(
                        Some(Cow::Borrowed(a_key)),
                        a_val,
                        b_val,
                    ));
                    continue;
                }
                if a_val.is_table() || a_val.is_array() {
                    stack.push((a_val, b_val));
                } else {
                    changes.push(TomlChange::Changed(
                        Some(Cow::Borrowed(a_key)),
                        a_val,
                        b_val,
                    ));
                }
            }
            todo!("Handle left-over key-value pairs")
        }
        Self { changes }
    }
}

#[cfg(test)]
mod test {
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
}
