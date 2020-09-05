use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[derive(Clone, Debug)]
pub struct Entry {
    pub topic: String,
    pub filename: String,
    pub weight: u8,
}

#[derive(Clone, Debug, Default)]
pub struct TagsDb {
    entries: HashMap<String, Vec<Entry>>,
}

fn trim_topic(topic: &str) -> &str {
    topic.trim_matches(|c: char| !c.is_alphanumeric())
}

fn add_entry(
    hm: &mut HashMap<String, Vec<Entry>>,
    key: &str,
    topic: &str,
    filename: &str,
    weight: u8,
) {
    let entry = Entry {
        topic: topic.to_owned(),
        filename: filename.to_owned(),
        weight,
    };
    hm.entry(key.to_owned()).or_default().push(entry);
}

fn best_match(opts: &[Entry]) -> Option<Entry> {
    let mut opts: Vec<_> = opts.to_owned();
    opts.sort_unstable_by_key(|e| e.weight);
    opts.get(0).map(Entry::clone)
}

impl TagsDb {
    pub fn read_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut entries: HashMap<_, Vec<_>> = HashMap::new();
        let f = BufReader::new(File::open(path)?);
        for line in f.lines() {
            let line = line?;
            let parts: Vec<_> = line.split('\t').take(2).collect();
            if parts.len() != 2 {
                log::warn!("Too few entries in tags line `{}`", line);
                continue;
            }
            let topic = parts[0];
            let filename = parts[1].trim_end_matches(".txt");
            add_entry(&mut entries, topic, topic, filename, 0);
            add_entry(&mut entries, trim_topic(topic), topic, filename, 1);
            let topic_lc = parts[0].to_ascii_lowercase();
            add_entry(&mut entries, &topic_lc, topic, filename, 2);
            add_entry(&mut entries, trim_topic(&topic_lc), topic, filename, 3);
        }
        Ok(Self { entries })
    }

    pub fn find(&self, topic: &str) -> Option<Entry> {
        let topic_lc = topic.to_lowercase();
        if let Some(result) = self.entries.get(topic).map(|xs| best_match(xs)) {
            result
        } else if let Some(result) = self.entries.get(trim_topic(topic)).map(|xs| best_match(xs)) {
            result
        } else if let Some(result) = self.entries.get(&topic_lc).map(|xs| best_match(xs)) {
            result
        } else if let Some(result) = self
            .entries
            .get(trim_topic(&topic_lc))
            .map(|xs| best_match(xs))
        {
            result
        } else {
            None
        }
    }
}
