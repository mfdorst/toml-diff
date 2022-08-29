use std::borrow::Cow;

use toml::Value as TomlValue;

pub enum TomlDiff<'a> {
    Same,
    Added(Cow<'a, str>, &'a TomlValue),
    Deleted(Cow<'a, str>, &'a TomlValue),
}

/// Return a list of differences between `a` and `b`
pub fn diff<'a>(a: &'a TomlValue, b: &'a TomlValue) -> Vec<TomlDiff<'a>> {
    // Recursive implementation of `diff`
    fn diff_rec<'a>(key: Cow<'a, str>, a: &'a TomlValue, b: &'a TomlValue) -> Vec<TomlDiff<'a>> {
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
                    if let (Some((a_key, a_val)), Some((b_key, b_val))) = (a_it.peek(), b_it.peek())
                    {
                        if a_key == b_key {
                            changes.extend(diff_rec(Cow::Owned(format!("{key}.{a_key}")), a_val, b_val));
                        } else {
                            // Keys are sorted low to high, so the lower order key is missing from the
                            // other table
                            if a_key < b_key {
                                changes.push(TomlDiff::Added(Cow::Borrowed(a_key), a_val));
                                a_it.next();
                            } else {
                                changes.push(TomlDiff::Deleted(Cow::Borrowed(b_key), b_val));
                                b_it.next();
                            }
                        }
                    } else {
                        // One of the iterators has ended.
                        // Anything left in a_it is an addition.
                        changes.extend(a_it.map(|(key, val)| TomlDiff::Added(Cow::Borrowed(key), val)));
                        // Anything left in b_it is a deletion.
                        changes.extend(b_it.map(|(key, val)| TomlDiff::Deleted(Cow::Borrowed(key), val)));
                        break;
                    }
                }

                changes
            }
            (TomlValue::Array(a_vec), TomlValue::Array(b_vec)) => {
                let mut changes = vec![];
                for (i, (a_elem, b_elem)) in a_vec.into_iter().zip(b_vec).enumerate() {
                    changes.extend(diff_rec(Cow::Owned(format!("{key}[{i}]")), a_elem, b_elem));
                }
                changes
            }
            (a, b) => {
                if a == b {
                    vec![TomlDiff::Same]
                } else {
                    // TODO: Figure out how to get rid of this clone
                    vec![TomlDiff::Added(key.clone(), a), TomlDiff::Deleted(key, b)]
                }
            }
        }
    }
    diff_rec(Cow::Borrowed(""), a, b)
}


