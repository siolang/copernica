use packets::{Interest, Data};

use lru::LruCache;
use bitvec::prelude::*;

pub struct SparseDistributedRouting {
    sdr: BitVec,
    lru: LruCache<String, Vec<Vec<usize>>>,
}

impl SparseDistributedRouting {
    pub fn new() -> Self {
        SparseDistributedRouting {
            sdr: bitvec![0; 2048],
            lru: LruCache::new(1000),
        }
    }

    pub fn insert(&mut self, interest: Interest) {
        let name: &String = &interest.name();
        let indices: &Vec<Vec<usize>> = &interest.index();
        self.lru.put(name.to_string(), indices.clone());
        for name in indices {
            for element in name {
                self.sdr.set(*element, true);
            }
        }
    }

    pub fn contains(&mut self, interest: Interest) -> u32 { //returns a percentage
        let mut vals: Vec<u32> = Vec::new();
        for name in interest.index() {
            for element in name {
                vals.push(self.sdr.get(*element).unwrap() as u32);
            }
        }
        let mut hits = vals.iter().try_fold(0u32, |acc, &elem| acc.checked_add(elem));
        let percentage = (hits.unwrap() as f32 / vals.len() as f32) * 100f32;
        //println!("hits: {:?}, length: {:?}, percentage: {}", hits.unwrap(), vals.len(), percentage);
        percentage as u32
    }

    pub fn print(&self) {
        println!("{:?}", self.sdr);
    }
}


#[cfg(test)]
mod sdr_tests {
    use super::*;

    #[test]
    fn contains_return_100_percent() {
        let interest = Interest::new("interested/in/world/affairs".to_string());
        let mut sdr = SparseDistributedRouting::new();
        sdr.insert(interest.clone());
        //println!("{}", sdr.contains(interest.clone()));
        //sdr.print();
        assert_eq!(sdr.contains(interest), 100);
    }
}
