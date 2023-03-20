use std::{iter::repeat_with, mem::size_of};

use crate::value::{hash, ThinString, Value};

#[derive(Default)]
pub struct Table {
    entries: Box<[Slot]>,
    count: usize,
}

impl Table {
    pub fn set(&mut self, key: String, value: Value) -> bool {
        if self.count * 4 >= self.capacity() * 3 {
            let new_capacity = if self.capacity() < 8 {
                8
            } else {
                self.capacity() * 2
            };
            self.realloc(new_capacity);
        }
        let entry = self.find_mut(&key);
        let is_new_key = !matches!(entry, Slot::Occupied(_));
        let was_tombstone = matches!(entry, Slot::Tombstone);
        *entry = Slot::Occupied(OccupiedEntry {
            key: ThinString::new(key),
            value,
        });
        if is_new_key && !was_tombstone {
            self.count += 1;
        }
        is_new_key
    }

    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        if self.count == 0 {
            return None;
        }
        match self.find(key) {
            Slot::Occupied(OccupiedEntry { value, .. }) => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        if self.count == 0 {
            return None;
        }
        match self.find_mut(key) {
            Slot::Occupied(OccupiedEntry { value, .. }) => Some(value),
            _ => None,
        }
    }

    pub fn delete(&mut self, key: &str) -> Option<Value> {
        if self.count == 0 {
            return None;
        }
        match self.find_mut(key) {
            entry @ Slot::Occupied(_) => {
                let entry = std::mem::replace(entry, Slot::Tombstone);
                let value = match entry {
                    Slot::Occupied(OccupiedEntry { value, .. }) => value,
                    _ => unreachable!(),
                };
                Some(value)
            }
            _ => None,
        }
    }

    // pub fn entry(&mut self, key: String) -> Entry {
    //     let slot = self.find_mut(&key);
    //     Entry { slot, key }
    // }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.entries.iter().filter_map(|x| match x {
            Slot::Occupied(OccupiedEntry { key, value }) => {
                Some((key.as_str(), value))
            }
            Slot::Vacant | Slot::Tombstone => None,
        })
    }

    fn realloc(&mut self, new_capacity: usize) {
        self.count = 0;
        let new_entries =
            repeat_with(|| Slot::Vacant).take(new_capacity).collect();
        let old_entries = std::mem::replace(&mut self.entries, new_entries);
        for entry in old_entries.into_vec() {
            if let Slot::Occupied(entry) = entry {
                self.count += 1;
                let dest = self.find_mut(&entry.key);
                *dest = Slot::Occupied(entry);
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
    fn find(&self, key: &str) -> &Slot {
        let mut index = hash(key.as_bytes()) % self.capacity() as u32;
        let mut tombstone = None;
        loop {
            let entry = &self.entries[index as usize];
            match entry {
                Slot::Occupied(OccupiedEntry { key: entry_key, .. })
                    if entry_key.as_str() != key =>
                {
                    ()
                }
                Slot::Tombstone => {
                    tombstone.get_or_insert(index);
                }
                Slot::Occupied(_) | Slot::Vacant => {
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
    fn find_mut(&mut self, key: &str) -> &mut Slot {
        let mut index = hash(key.as_bytes()) % self.capacity() as u32;
        let mut tombstone = None;
        loop {
            let entry = &mut self.entries[index as usize];
            match entry {
                Slot::Occupied(OccupiedEntry { key: entry_key, .. })
                    if entry_key.as_str() != key =>
                {
                    ()
                }
                Slot::Tombstone => {
                    tombstone.get_or_insert(index);
                }
                Slot::Occupied(_) | Slot::Vacant => {
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

// pub struct Entry<'map> {
//     slot: &'map mut Slot,
//     key: String,
// }

// impl Entry<'_> {
//     pub fn set_if_empty(self, new_value: Value) -> bool {
//         match self.slot {
//             Slot::Occupied(OccupiedEntry { value, .. }) => {
//                 *value = new_value;
//                 true
//             }
//             Slot::Vacant | Slot::Tombstone => false,
//         }
//     }
// }

enum Slot {
    Occupied(OccupiedEntry),
    Vacant,
    Tombstone,
}

struct OccupiedEntry {
    key: ThinString,
    value: Value,
}

// Entry can just reuse Value's tag niches for its tag
const _: () = assert!(size_of::<Slot>() == size_of::<OccupiedEntry>());
