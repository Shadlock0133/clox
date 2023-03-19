use std::{iter::repeat_with, mem::size_of};

use crate::value::{hash, ThinString, Value};

#[derive(Default)]
pub struct Table {
    entries: Box<[Entry]>,
    count: usize,
}

impl Table {
    pub fn set(&mut self, key: String, value: Value) -> bool {
        if self.count * 4 > self.capacity() * 3 {
            let new_capacity = if self.capacity() < 8 {
                8
            } else {
                self.capacity() * 2
            };
            self.realloc(new_capacity);
        }
        let entry = self.find_mut(&key);
        let is_new_key = !matches!(entry, Entry::Occupied(_));
        let was_tombstone = matches!(entry, Entry::Tombstone);
        *entry = Entry::Occupied(OccupiedEntry {
            key: ThinString::new(key),
            value,
        });
        if is_new_key && !was_tombstone {
            self.count += 1;
        }
        is_new_key
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        if self.count == 0 {
            return None;
        }
        match self.find(key) {
            Entry::Occupied(OccupiedEntry { value, .. }) => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        if self.count == 0 {
            return None;
        }
        match self.find_mut(key) {
            Entry::Occupied(OccupiedEntry { value, .. }) => Some(value),
            _ => None,
        }
    }

    pub fn delete(&mut self, key: &str) -> Option<Value> {
        if self.count == 0 {
            return None;
        }
        match self.find_mut(key) {
            entry @ Entry::Occupied(_) => {
                let entry = std::mem::replace(entry, Entry::Tombstone);
                let value = match entry {
                    Entry::Occupied(OccupiedEntry { value, .. }) => value,
                    _ => unreachable!(),
                };
                Some(value)
            }
            _ => None,
        }
    }

    fn realloc(&mut self, new_capacity: usize) {
        self.count = 0;
        let new_entries =
            repeat_with(|| Entry::Vacant).take(new_capacity).collect();
        let old_entries = std::mem::replace(&mut self.entries, new_entries);
        for entry in old_entries.into_vec() {
            if let Entry::Occupied(entry) = entry {
                self.count += 1;
                let dest = self.find_mut(&entry.key);
                *dest = Entry::Occupied(entry);
            }
        }
    }

    fn capacity(&self) -> usize {
        self.entries.len()
    }

    // returns either (in that order):
    // - occupied entry with same key
    // - first tombstone slot
    // - vacant slot
    fn find(&self, key: &str) -> &Entry {
        let mut index = hash(key.as_bytes()) % self.capacity() as u32;
        let mut tombstone = None;
        loop {
            let entry = &self.entries[index as usize];
            match entry {
                Entry::Occupied(OccupiedEntry { key: entry_key, .. })
                    if entry_key.as_str() != key =>
                {
                    ()
                }
                Entry::Tombstone => {
                    tombstone.get_or_insert(index);
                }
                Entry::Occupied(_) | Entry::Vacant => {
                    if let Some(index) = tombstone {
                        return &self.entries[index as usize];
                    }
                    return entry;
                }
            }
            index = (index + 1) % self.capacity() as u32;
        }
    }

    // same as `find`
    fn find_mut(&mut self, key: &str) -> &mut Entry {
        let mut index = hash(key.as_bytes()) % self.capacity() as u32;
        let mut tombstone = None;
        loop {
            let entry = &mut self.entries[index as usize];
            match entry {
                Entry::Occupied(OccupiedEntry { key: entry_key, .. })
                    if entry_key.as_str() != key =>
                {
                    ()
                }
                Entry::Tombstone => {
                    tombstone.get_or_insert(index);
                }
                Entry::Occupied(_) | Entry::Vacant => {
                    if let Some(index) = tombstone {
                        return &mut self.entries[index as usize];
                    }
                    return &mut self.entries[index as usize];
                }
            }
            index = (index + 1) % self.capacity() as u32;
        }
    }
}

impl Extend<(String, Value)> for Table {
    fn extend<T: IntoIterator<Item = (String, Value)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.set(key, value);
        }
    }
}

enum Entry {
    Occupied(OccupiedEntry),
    Vacant,
    Tombstone,
}

struct OccupiedEntry {
    key: ThinString,
    value: Value,
}

// Entry can just reuse Value's tag niches for its tag
const _: () = assert!(size_of::<Entry>() == size_of::<OccupiedEntry>());
