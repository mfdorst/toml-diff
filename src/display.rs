use std::borrow::Cow;
use std::fmt;

use toml::Value as TomlValue;

use crate::{TomlChange, TomlDiff};

impl<'a> fmt::Display for TomlDiff<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for change in &self.changes {
            match change {
                TomlChange::Same => Ok(()),
                // TODO: Don't clone
                TomlChange::Added(key, val) => format_change(f, '+', key.clone(), val),
                TomlChange::Deleted(key, val) => format_change(f, '-', key.clone(), val),
                TomlChange::Changed(_key, _val_a, _val_b) => {
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
        TomlValue::String(val) => writeln!(f, "{prefix} {key} = \"{val}\""),
        TomlValue::Table(table) => {
            writeln!(f, "{prefix} [{key}]")?;
            for (key, val) in table {
                format_change(f, prefix, Cow::Borrowed(key), val)?;
            }
            Ok(())
        }
        val => {
            // TODO: Don't unwrap
            let serialized = toml::to_string(val).unwrap();
            // TODO: Colorize
            writeln!(f, "{prefix} {key} = {serialized}")
        }
    }
}
