use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::PrimaryKey;
use cw_storage_plus::{Bound, Key, KeyDeserialize, Path, Prefix};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

pub struct NonceMap<'a, K, T> {
    namespace: &'a [u8],
    key_type: PhantomData<K>,
    data_type: PhantomData<T>,
}

impl<'a, K, T> NonceMap<'a, K, T> {
    pub const fn new(namespace: &'a str) -> Self {
        NonceMap {
            namespace: namespace.as_bytes(),
            data_type: PhantomData,
            key_type: PhantomData,
        }
    }
}

impl<'a, K, T> NonceMap<'a, K, T>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a>,
{
    fn key(&self, nonce: u128, k: K) -> Path<T> {
        // Add raw nonce bytes (without length prefix)
        let nonce_bytes = nonce.to_be_bytes();
        let mut keys: Vec<&[u8]> = vec![&nonce_bytes];

        // Add Keys
        let binding = k.key();
        keys.extend(binding.iter().map(Key::as_ref));

        Path::new(self.namespace, &keys)
    }

    fn no_prefix_raw(&self, nonce: u128) -> Prefix<Vec<u8>, T> {
        Prefix::new(self.namespace, &[Key::Ref(&nonce.to_be_bytes())])
    }

    // Retrieves permissions from storage and returns None when missing
    // Doesn't take inherited permissions into account.
    pub fn load(&self, store: &dyn Storage, nonce: u128, k: K) -> StdResult<Option<T>> {
        self.key(nonce, k).may_load(store)
    }

    pub fn save(&self, store: &mut dyn Storage, nonce: u128, k: K, data: &T) -> StdResult<()> {
        self.key(nonce, k).save(store, data)
    }

    pub fn remove(&self, store: &mut dyn Storage, nonce: u128, k: K) {
        self.key(nonce, k).remove(store)
    }

    pub fn has(&self, store: &dyn Storage, nonce: u128, k: K) -> bool {
        self.key(nonce, k).has(store)
    }

    pub fn clear(&self, store: &mut dyn Storage, nonce: u128) {
        const TAKE: usize = 10;
        let mut cleared = false;

        while !cleared {
            let paths = self
                .no_prefix_raw(nonce)
                .keys_raw(store, None, None, cosmwasm_std::Order::Ascending)
                .map(|raw_key| {
                    Path::<T>::new(self.namespace, &[&nonce.to_be_bytes(), raw_key.as_slice()])
                })
                // Take just TAKE elements to prevent possible heap overflow if the Map is big.
                .take(TAKE)
                .collect::<Vec<_>>();

            paths.iter().for_each(|path| store.remove(path));

            cleared = paths.len() < TAKE;
        }
    }
}

impl<'a, K, T> NonceMap<'a, K, T>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + KeyDeserialize,
{
    fn no_prefix(&self, nonce: u128) -> Prefix<K, T, K> {
        Prefix::new(self.namespace, &[Key::Ref(&nonce.to_be_bytes())])
    }

    pub fn range<'c>(
        &self,
        store: &'c dyn Storage,
        nonce: u128,
        min: Option<Bound<'a, K>>,
        max: Option<Bound<'a, K>>,
        order: cosmwasm_std::Order,
    ) -> Box<dyn Iterator<Item = StdResult<(K::Output, T)>> + 'c>
    where
        T: 'c,
        K::Output: 'static,
    {
        self.no_prefix(nonce).range(store, min, max, order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::Order;

    const TEST_MAP: NonceMap<String, String> = NonceMap::new("test");

    #[test]
    fn clear_works() {
        let mut deps = mock_dependencies();
        // Addresses

        let nonce = 1u128;

        assert!(TEST_MAP
            .save(
                deps.as_mut().storage,
                nonce,
                "ABCD".to_string(),
                &"1".to_string(),
            )
            .is_ok());

        assert!(TEST_MAP
            .save(
                deps.as_mut().storage,
                nonce,
                "ABCDF".to_string(),
                &"3".to_string(),
            )
            .is_ok());

        assert!(TEST_MAP
            .save(
                deps.as_mut().storage,
                nonce,
                "ABCDE".to_string(),
                &"2".to_string(),
            )
            .is_ok());

        let res: Vec<(String, String)> = TEST_MAP
            .range(deps.as_ref().storage, nonce, None, None, Order::Ascending)
            .map(|data| data.unwrap())
            .collect();

        assert_eq!(res[0], ("ABCD".to_string(), "1".to_string()));
        assert_eq!(res[1], ("ABCDE".to_string(), "2".to_string()));
        assert_eq!(res[2], ("ABCDF".to_string(), "3".to_string()));
        assert_eq!(res.len(), 3);

        TEST_MAP.clear(deps.as_mut().storage, nonce);

        let res: Vec<(String, String)> = TEST_MAP
            .range(deps.as_ref().storage, nonce, None, None, Order::Ascending)
            .map(|data| data.unwrap())
            .collect();

        assert_eq!(res.len(), 0);
    }
}
