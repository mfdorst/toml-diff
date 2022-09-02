use std::{borrow::Cow, fmt};

use toml::Value as TomlValue;

use crate::{TomlChange, TomlDiff};

impl<'a> fmt::Display for TomlDiff<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for change in &self.changes {
            match change {
                TomlChange::Same => Ok(()),
                TomlChange::Added(key, val) => write!(f, "{}", format_change('+', key.clone(), val)?),
                TomlChange::Deleted(key, val) => write!(f, "{}", format_change('-', key.clone(), val)?),
            }?;
        }
        Ok(())
    }
}

fn format_change<'a>(
    prefix: char,
    key: Option<Cow<'a, str>>,
    val: &'a TomlValue,
) -> Result<String, fmt::Error> {
    let s = match key {
        Some(key) => {
            let mut wrapper = toml::map::Map::new();
            wrapper.insert(key.into_owned(), val.clone());
            toml::to_string(&wrapper)
        }
        None => toml::to_string(val),
    }.map_err(|_| fmt::Error)?;
    // Prepend the prefix to each line
    Ok(s.lines()
        .map(|line| format!("{prefix} {line}\n"))
        .collect::<Vec<_>>()
        .join(""))
}

