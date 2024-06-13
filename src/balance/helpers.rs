use cosmwasm_std::{Coin, StdError, StdResult, Uint128};
use std::collections::btree_map::Entry;
use std::collections::btree_map::OccupiedEntry;
use std::collections::BTreeMap;

pub trait BTreeMapCoinHelpers {
    fn into_vec(self) -> Vec<Coin>;
    fn inplace_sub<'a, I>(&mut self, balance: I) -> StdResult<()>
    where
        I: IntoIterator<Item = (&'a String, &'a Uint128)>;

    fn inplace_add<'a, I>(&mut self, balance: I) -> StdResult<()>
    where
        I: IntoIterator<Item = (&'a String, &'a Uint128)>;
}
impl BTreeMapCoinHelpers for BTreeMap<String, Uint128> {
    fn into_vec(self) -> Vec<Coin> {
        self.into_iter()
            .map(|(denom, amount)| Coin { denom, amount })
            .collect()
    }

    fn inplace_sub<'a, I>(&mut self, balance: I) -> StdResult<()>
    where
        I: IntoIterator<Item = (&'a String, &'a Uint128)>,
    {
        // Decrease total remaining supply
        for (denom, amount) in balance {
            if let Some(r) = self.get_mut(denom) {
                if amount > r {
                    return Err(StdError::generic_err(format!(
                        "Subtract overflow for denom: {}",
                        denom
                    )));
                }
                *r -= amount;
            } else {
                return Err(StdError::generic_err(format!("Unknown denom {}", denom)));
            }

            // Remove denom if balance is now 0
            if self.get(denom) == Some(&Uint128::zero()) {
                self.remove(denom);
            }
        }
        Ok(())
    }

    fn inplace_add<'a, I>(&mut self, balance: I) -> StdResult<()>
    where
        I: IntoIterator<Item = (&'a String, &'a Uint128)>,
    {
        for (denom, amount) in balance {
            if let Some(counter) = self.get_mut(denom) {
                *counter = counter.checked_add(*amount).map_err(|_| {
                    StdError::generic_err(format!("Addition overflow for denom: {}", denom))
                })?;
            } else {
                self.insert(denom.clone(), *amount);
            }
        }
        Ok(())
    }
}

pub trait VecCoinConversions {
    fn to_tuple_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a String, &'a Uint128)> + 'a>;
    fn into_map_with_duplicities<F>(
        self,
        handle_duplicate: F,
    ) -> StdResult<BTreeMap<String, Uint128>>
    where
        F: Fn(OccupiedEntry<String, Uint128>, Uint128) -> StdResult<()>;
    fn into_map(self) -> StdResult<BTreeMap<String, Uint128>>;
    fn to_formatted_string(&self) -> String;
}

impl VecCoinConversions for Vec<Coin> {
    fn to_tuple_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a String, &'a Uint128)> + 'a> {
        Box::new(self.iter().map(|coin| (&coin.denom, &coin.amount)))
    }

    fn into_map_with_duplicities<F>(
        self,
        handle_duplicate: F,
    ) -> StdResult<BTreeMap<String, Uint128>>
    where
        F: Fn(OccupiedEntry<String, Uint128>, Uint128) -> StdResult<()>,
    {
        let mut denom_map = BTreeMap::new();

        for coin in self {
            match denom_map.entry(coin.denom) {
                Entry::Vacant(e) => {
                    e.insert(coin.amount);
                }
                Entry::Occupied(e) => {
                    handle_duplicate(e, coin.amount)?;
                }
            }
        }

        Ok(denom_map)
    }

    fn into_map(self) -> StdResult<BTreeMap<String, Uint128>> {
        self.into_map_with_duplicities(|e, _| {
            Err(StdError::generic_err(format!(
                "Duplicate denom found: {}",
                e.key()
            )))
        })
    }

    fn to_formatted_string(&self) -> String {
        self.iter()
            .map(|coin| format!("{}{}", coin.amount, coin.denom))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coin, Uint128};
    use std::collections::BTreeMap;

    #[test]
    fn test_into_vec() {
        let mut map = BTreeMap::new();
        map.insert("atom".to_string(), Uint128::new(100));
        map.insert("btc".to_string(), Uint128::new(50));

        let vec = map.into_vec();
        assert_eq!(vec, vec![coin(100, "atom"), coin(50, "btc")]);
    }

    #[test]
    fn test_inplace_sub() {
        let mut map = BTreeMap::new();
        map.insert("atom".to_string(), Uint128::new(100));
        map.insert("btc".to_string(), Uint128::new(50));

        let balance = vec![
            ("atom".to_string(), Uint128::new(30)),
            ("btc".to_string(), Uint128::new(20)),
        ];
        map.inplace_sub(balance.iter().map(|(d, a)| (d, a)))
            .unwrap();

        assert_eq!(map.get("atom"), Some(&Uint128::new(70)));
        assert_eq!(map.get("btc"), Some(&Uint128::new(30)));
    }

    #[test]
    fn test_inplace_sub_overflow() {
        let mut map = BTreeMap::new();
        map.insert("atom".to_string(), Uint128::new(100));

        let balance = vec![("atom".to_string(), Uint128::new(150))];
        let result = map.inplace_sub(balance.iter().map(|(d, a)| (d, a)));

        assert_eq!(
            result.err(),
            Some(StdError::generic_err("Subtract overflow for denom: atom"))
        );
    }

    #[test]
    fn test_inplace_add() {
        let mut map = BTreeMap::new();
        map.insert("atom".to_string(), Uint128::new(100));
        map.insert("btc".to_string(), Uint128::new(50));

        let balance = vec![
            ("atom".to_string(), Uint128::new(30)),
            ("btc".to_string(), Uint128::new(20)),
            ("eth".to_string(), Uint128::new(10)),
        ];
        assert!(map.inplace_add(balance.iter().map(|(d, a)| (d, a))).is_ok());

        assert_eq!(map.get("atom"), Some(&Uint128::new(130)));
        assert_eq!(map.get("btc"), Some(&Uint128::new(70)));
        assert_eq!(map.get("eth"), Some(&Uint128::new(10)));
    }

    #[test]
    fn test_to_tuple_iterator() {
        let vec = vec![coin(100, "atom"), coin(50, "btc")];
        let mut iter = vec.to_tuple_iterator();

        assert_eq!(iter.next(), Some((&"atom".to_string(), &Uint128::new(100))));
        assert_eq!(iter.next(), Some((&"btc".to_string(), &Uint128::new(50))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_map() {
        let vec = vec![coin(100, "atom"), coin(50, "btc"), coin(50, "btc")];
        let map = vec
            .into_map_with_duplicities(|mut e, new_value| Ok(*e.get_mut() += new_value))
            .unwrap();

        assert_eq!(map.get("atom"), Some(&Uint128::new(100)));
        assert_eq!(map.get("btc"), Some(&Uint128::new(100)));
    }

    #[test]
    fn test_to_formatted_string() {
        let vec = vec![coin(100, "atom"), coin(50, "btc")];
        let formatted = vec.to_formatted_string();

        assert_eq!(formatted, "100atom, 50btc");
    }

    #[test]
    fn test_empty_map_add_subtract() {
        let mut map = BTreeMap::new();

        let balance = vec![coin(30, "atom"), coin(20, "btc"), coin(10, "eth")];
        assert!(map.inplace_add(balance.to_tuple_iterator()).is_ok());

        assert_eq!(map.get("atom"), Some(&Uint128::new(30)));
        assert_eq!(map.get("btc"), Some(&Uint128::new(20)));
        assert_eq!(map.get("eth"), Some(&Uint128::new(10)));

        assert!(map.inplace_sub(balance.to_tuple_iterator()).is_ok());

        assert!(map.is_empty())
    }
    #[test]
    fn test_subtracting_nonexistent_denom() {
        let mut map = BTreeMap::new();
        map.insert("atom".to_string(), Uint128::new(100));

        let balance = vec![("btc".to_string(), Uint128::new(50))];
        let result = map.inplace_sub(balance.iter().map(|(d, a)| (d, a)));

        assert!(result.is_err());
        assert_eq!(
            result.err(),
            Some(StdError::generic_err("Unknown denom btc"))
        );
    }

    #[test]
    fn test_addition_with_overflow() {
        let mut map = BTreeMap::new();
        map.insert("atom".to_string(), Uint128::new(u128::MAX));

        let balance = vec![("atom".to_string(), Uint128::new(1))];
        let result = map.inplace_add(balance.iter().map(|(d, a)| (d, a)));

        assert_eq!(
            result.err(),
            Some(StdError::generic_err("Addition overflow for denom: atom"))
        );

        assert_eq!(map.get("atom"), Some(&Uint128::new(u128::MAX)));
    }

    #[test]
    fn test_ordering() {
        let coins = vec![
            coin(50, "btc"),
            coin(100, "atom"),
            coin(200, "eth"),
            coin(150, "usd"),
            coin(75, "eur"),
        ];

        let sorted_coins = vec![
            coin(100, "atom"),
            coin(50, "btc"),
            coin(200, "eth"),
            coin(75, "eur"),
            coin(150, "usd"),
        ];
        let sorted_keys: Vec<String> = sorted_coins.iter().map(|res| res.denom.clone()).collect();

        // Convert vector of coins into map
        let map = coins.into_map().unwrap();

        // Ensure that keys in map are sorted
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys, sorted_keys);

        // Ensure that vec output is sorted
        assert_eq!(map.into_vec(), sorted_coins);
    }

    #[test]
    fn test_vec_of_coins_into_btreemap_unique() {
        let coins = vec![
            coin(100, "atom"),
            coin(50, "btc"),
            coin(200, "eth"),
            coin(150, "usd"),
            coin(75, "eur"),
        ];

        // Resulting maps are equivalent if there is no duplicity
        assert_eq!(
            coins.clone().into_map().unwrap(),
            coins
                .clone()
                .into_map_with_duplicities(|mut e, b| { Ok(*e.get_mut() = b) })
                .unwrap()
        );

        let result = coins.into_map();
        assert!(result.is_ok());

        let map = result.unwrap();
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(
            keys,
            vec![
                "atom".to_string(),
                "btc".to_string(),
                "eth".to_string(),
                "eur".to_string(),
                "usd".to_string()
            ]
        );

        assert_eq!(map.get("atom"), Some(&Uint128::new(100)));
        assert_eq!(map.get("btc"), Some(&Uint128::new(50)));
        assert_eq!(map.get("eth"), Some(&Uint128::new(200)));
        assert_eq!(map.get("usd"), Some(&Uint128::new(150)));
        assert_eq!(map.get("eur"), Some(&Uint128::new(75)));
    }

    #[test]
    fn test_vec_of_coins_into_btreemap_unique_with_duplicates() {
        let coins = vec![
            coin(100, "atom"),
            coin(50, "btc"),
            coin(200, "eth"),
            coin(100, "btc"), // Duplicate denom
            coin(75, "eth"),  // Duplicate denom
        ];

        let result = coins.into_map();
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err, StdError::generic_err("Duplicate denom found: btc"));
        }
    }

    #[test]
    fn test_vec_of_coins_into_btreemap_unique_with_empty_vec() {
        let coins: Vec<Coin> = vec![];

        let result = coins.into_map();
        assert!(result.is_ok());

        let map = result.unwrap();
        assert!(map.is_empty());
    }
}
