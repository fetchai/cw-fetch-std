use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Key, Path, Prefix, PrimaryKey};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct StorageSet<'a, K> {
    namespace: &'a [u8],
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    key_type: PhantomData<K>,
}

impl<'a, K> StorageSet<'a, K> {
    pub const fn new(namespace: &'a str) -> Self {
        StorageSet {
            namespace: namespace.as_bytes(),
            key_type: PhantomData,
        }
    }
}

impl<'a, K> StorageSet<'a, K>
where
    K: PrimaryKey<'a>,
{
    pub fn key(&self, k: &K) -> Path<()> {
        Path::new(
            self.namespace,
            &k.key().iter().map(Key::as_ref).collect::<Vec<_>>(),
        )
    }

    pub fn has(&self, store: &dyn Storage, key: &K) -> bool {
        self.key(key).has(store)
    }

    pub fn add(&self, storage: &mut dyn Storage, key: &K) -> StdResult<()> {
        self.key(key).save(storage, &())
    }

    pub fn remove(&self, store: &mut dyn Storage, key: &K) {
        self.key(key).remove(store)
    }
}

impl<'a, K> StorageSet<'a, K>
where
    K: PrimaryKey<'a> + cw_storage_plus::KeyDeserialize<Output = K> + 'static,
{
    pub fn get_all(&self, store: &dyn Storage) -> StdResult<Vec<K>> {
        Ok(self
            .no_prefix()
            .range(store, None, None, Order::Ascending)
            .map(|data| data.unwrap().0)
            .collect())
    }

    fn no_prefix(&self) -> Prefix<K, (), K> {
        Prefix::new(self.namespace, &[])
    }
}
