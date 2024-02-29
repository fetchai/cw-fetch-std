use cosmwasm_std::{StdResult, Storage};

pub trait Parentable
where
    Self: Sized + Clone,
{
    // Return parent
    fn parent(&self) -> Option<Self>;
}

// Returns the first non-None result of type Value while going up the tree.
// Returns None if no result is found.
// F is a function that reads from storage using Parentable struct like DomainKey and returns an Option<Value>
pub fn get_inherited<K, V, F>(
    storage: &dyn Storage,
    domain: K,
    mut store_get: F,
) -> StdResult<Option<V>>
where
    F: FnMut(&dyn Storage, &K) -> StdResult<Option<V>>,
    K: Parentable + Clone,
{
    let mut current_comain = domain;

    // Iterate from the parent domains to the root domain using a while loop
    loop {
        if let Some(result) = store_get(storage, &current_comain)? {
            // Found a result
            return Ok(Some(result));
        }
        // Go to the next parent domain
        current_comain = match current_comain.parent() {
            Some(parent) => parent,
            None => return Ok(None),
        }
    }
}
