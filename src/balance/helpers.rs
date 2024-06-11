use cosmwasm_std::{Coin, StdError, StdResult, Uint128};
use std::collections::BTreeMap;

pub trait BTreeMapCoinHelpers {
    fn into_vec(self) -> Vec<Coin>;
    fn inplace_sub<'a, I>(&mut self, balance: I) -> StdResult<()>
    where
        I: IntoIterator<Item = (&'a String, &'a Uint128)>;

    fn inplace_add<'a, I>(&mut self, balance: I)
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

    fn inplace_add<'a, I>(&mut self, balance: I)
    where
        I: IntoIterator<Item = (&'a String, &'a Uint128)>,
    {
        for (denom, amount) in balance {
            if let Some(counter) = self.get_mut(denom) {
                *counter += amount;
            } else {
                self.insert(denom.clone(), *amount);
            }
        }
    }
}

pub trait VecCoinConversions {
    fn to_tuple_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a String, &'a Uint128)> + 'a>;
    fn into_map(self) -> BTreeMap<String, Uint128>;
    fn to_formatted_string(&self) -> String;
}

impl VecCoinConversions for Vec<Coin> {
    fn to_tuple_iterator<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a String, &'a Uint128)> + 'a> {
        Box::new(self.iter().map(|coin| (&coin.denom, &coin.amount)))
    }

    fn into_map(self) -> BTreeMap<String, Uint128> {
        let mut denom_map = BTreeMap::new();

        for coin in self {
            if let Some(counter) = denom_map.get_mut(&coin.denom) {
                *counter += coin.amount;
            } else {
                denom_map.insert(coin.denom, coin.amount);
            }
        }

        denom_map
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

        assert!(result.is_err());
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
        map.inplace_add(balance.iter().map(|(d, a)| (d, a)));

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
        let map = vec.into_map();

        assert_eq!(map.get("atom"), Some(&Uint128::new(100)));
        assert_eq!(map.get("btc"), Some(&Uint128::new(100)));
    }

    #[test]
    fn test_to_formatted_string() {
        let vec = vec![coin(100, "atom"), coin(50, "btc")];
        let formatted = vec.to_formatted_string();

        assert_eq!(formatted, "100atom, 50btc");
    }
}
