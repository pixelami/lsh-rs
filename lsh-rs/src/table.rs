use crate::hash::Hash;
use crate::utils::all_eq;
use fnv::FnvHashMap as HashMap;
use fnv::FnvHashSet as HashSet;

pub type DataPoint = Vec<f32>;
pub type DataPointSlice = [f32];
/// Bucket contains indexes to VecStore
pub type Bucket = HashSet<u32>;
pub enum HashTableError {
    Failed,
    NotFound,
}

/// Indexible vector storage.
/// indexes will be stored in hashtables. The original vectors can be looked up in this data structure.
struct VecStore {
    map: Vec<DataPoint>,
}

impl VecStore {
    fn push(&mut self, d: DataPoint) -> u32 {
        self.map.push(d);
        (self.map.len() - 1) as u32
    }

    fn position(&self, d: &DataPointSlice) -> Option<u32> {
        self.map.iter().position(|x| all_eq(x, d)).map(|x| x as u32)
    }

    fn get(&self, idx: u32) -> &DataPoint {
        &self.map[idx as usize]
    }

    fn increase_storage(&mut self, size: usize) {
        if self.map.capacity() < size {
            let diff = size - self.map.capacity();
            self.map.reserve(diff)
        }
    }
}

/// Hashtable consisting of `L` Hash tables.
pub trait HashTables {
    /// # Arguments
    ///
    /// * `hash` - hashed vector.
    /// * `d` - Vector to store in the buckets.
    /// * `hash_table` - Number of the hash_table to store the vector. Ranging from 0 to L.
    fn put(&mut self, hash: Hash, d: DataPoint, hash_table: usize) -> Result<(), HashTableError>;

    fn delete(
        &mut self,
        hash: Hash,
        d: &DataPointSlice,
        hash_table: usize,
    ) -> Result<(), HashTableError>;

    /// Query the whole bucket
    fn query_bucket(&self, hash: &Hash, hash_table: usize) -> Result<&Bucket, HashTableError>;

    /// Query the most similar
    fn query(&self, distance_fn: &dyn Fn(DataPoint) -> f32) -> Result<DataPoint, HashTableError>;

    fn idx_to_datapoint(&self, idx: u32) -> &DataPoint;

    fn increase_storage(&mut self, size: usize);
}

pub struct MemoryTable {
    hash_tables: Vec<HashMap<Hash, Bucket>>,
    n_hash_tables: usize,
    vec_store: VecStore,
}

impl MemoryTable {
    pub fn new(n_hash_tables: usize) -> MemoryTable {
        // TODO: Check the average number of vectors in the buckets.
        // this way the capacity can be approximated by the number of DataPoints that will
        // be stored.
        let hash_tables = vec![HashMap::default(); n_hash_tables];
        let vector_store = VecStore { map: vec![] };
        MemoryTable {
            hash_tables,
            n_hash_tables,
            vec_store: vector_store,
        }
    }
}

impl HashTables for MemoryTable {
    fn put(&mut self, hash: Hash, d: DataPoint, hash_table: usize) -> Result<(), HashTableError> {
        let tbl = &mut self.hash_tables[hash_table];
        let bucket = tbl.entry(hash).or_insert_with(|| HashSet::default());
        let idx = self.vec_store.push(d);
        bucket.insert(idx);
        Ok(())
    }

    /// Expensive operation we need to do a linear search over all datapoints
    fn delete(
        &mut self,
        hash: Hash,
        d: &DataPointSlice,
        hash_table: usize,
    ) -> Result<(), HashTableError> {
        // First find the data point in the VecStore
        let idx = match self.vec_store.position(d) {
            None => return Ok(()),
            Some(idx) => idx,
        };
        // Note: data point remains in VecStore as shrinking the vector would mean we need to
        // re-hash all datapoints.

        // Then remove idx from hash tables
        let tbl = &mut self.hash_tables[hash_table];
        let bucket = tbl.get_mut(&hash);
        match bucket {
            None => return Err(HashTableError::NotFound),
            Some(bucket) => {
                bucket.remove(&idx);
                Ok(())
            }
        }
    }

    /// Query the whole bucket
    fn query_bucket(&self, hash: &Hash, hash_table: usize) -> Result<&Bucket, HashTableError> {
        let tbl = &self.hash_tables[hash_table];
        match tbl.get(hash) {
            None => Err(HashTableError::NotFound),
            Some(bucket) => Ok(bucket),
        }
    }

    /// Query the most similar
    fn query(&self, distance_fn: &dyn Fn(DataPoint) -> f32) -> Result<DataPoint, HashTableError> {
        Err(HashTableError::Failed)
    }

    fn idx_to_datapoint(&self, idx: u32) -> &DataPoint {
        self.vec_store.get(idx)
    }

    fn increase_storage(&mut self, size: usize) {
        self.vec_store.increase_storage(size);
    }
}

impl std::fmt::Debug for MemoryTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hash_tables:\nhash, \t buckets\n")?;
        for ht in self.hash_tables.iter() {
            write!(f, "{:?}\n", ht)?;
        }
        Ok(())
    }
}
