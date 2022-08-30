use std::borrow::Cow;
use std::cmp::Ordering;
use std::mem::discriminant;

use toml::Value as TomlValue;

mod display;
#[cfg(test)]
mod test;

pub struct TomlDiff<'a> {
    changes: Vec<TomlChange<'a>>,
}

pub enum TomlChange<'a> {
    Same,
    Added(Option<Cow<'a, str>>, &'a TomlValue),
    Deleted(Option<Cow<'a, str>>, &'a TomlValue),
    Changed(Option<Cow<'a, str>>, &'a TomlValue, &'a TomlValue),
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
                let mut a_it = a_vec.iter();
                let mut b_it = b_vec.iter();

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
                // Anything left in `a_it` is an addition (doesn't exist in `b`), and vice versa
                changes.extend(a_it.map(|e| TomlChange::Added(None, e)));
                changes.extend(b_it.map(|e| TomlChange::Deleted(None, e)));
                continue;
            }
            // We only ever push `Array`s or `Table`s to `stack`
            let a_map = a.as_table().unwrap();
            let b_map = b.as_table().unwrap();
            let mut a_pairs: Vec<_> = a_map.iter().collect();
            let mut b_pairs: Vec<_> = b_map.iter().collect();
            a_pairs.sort_by_key(|e| e.0);
            b_pairs.sort_by_key(|e| e.0);
            let mut a_pairs_it = a_pairs.into_iter().peekable();
            let mut b_pairs_it = b_pairs.into_iter().peekable();

            while let (Some((&ref a_key, &ref a_val)), Some((&ref b_key, &ref b_val))) =
                (a_pairs_it.peek(), b_pairs_it.peek())
            {
                // Keys are sorted low to high, so if the keys are different, that means
                // that the lesser key is missing from the other table.
                match a_key.cmp(b_key) {
                    Ordering::Less => {
                        // Keys missing from `b` are considdered "added" in `a`
                        changes.push(TomlChange::Added(Some(Cow::Borrowed(a_key)), a_val));
                        a_pairs_it.next();
                        continue;
                    }
                    Ordering::Greater => {
                        // Keys missing from `a` are considered "deleted" from `b`
                        changes.push(TomlChange::Deleted(Some(Cow::Borrowed(b_key)), b_val));
                        b_pairs_it.next();
                        continue;
                    }
                    Ordering::Equal => {
                        a_pairs_it.next();
                        b_pairs_it.next();
                    }
                }
                if a_val == b_val {
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
            // Anything left over in `a_pairs_it` is an addition (doesn't exist in `b`) and vice versa
            changes.extend(a_pairs_it.map(|(k, v)| TomlChange::Added(Some(Cow::Borrowed(k)), v)));
            changes.extend(b_pairs_it.map(|(k, v)| TomlChange::Deleted(Some(Cow::Borrowed(k)), v)));
        }
        Self { changes }
    }
}
