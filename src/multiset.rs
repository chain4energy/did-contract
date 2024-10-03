use cw_storage_plus::{Bound, Map, Namespace, Path, Prefix};
use cosmwasm_std::{StdResult, Storage};

/// MultiMap struct that provides an abstraction over the Path-based multi-map
pub struct MultiSet {
    // Name for the map, used as a base for constructing key paths
    namespace: &'static str,
}

impl MultiSet {
    // Create a new MultiMap with the given name
    pub const fn new(namespace: &'static str) -> Self {
        MultiSet { namespace: namespace }
    }

    // Create a Path using the base name, primary key, and secondary key
    fn create_submap_key(&self, primary_key: &str) -> String {
        let mut r = self.namespace.to_string();
        r.push_str(primary_key);
        r
    }

    // Add a value to the multi-map under the composite key (primary_key, secondary_key)
    pub fn save(
        &self,
        storage: &mut dyn Storage,
        primary_key: &str,
        value: &str
    ) -> Result<(), cosmwasm_std::StdError>  {
        let k = self.create_submap_key(primary_key);
        let map: Map<String, bool> = Map::new_dyn(k);
        map.save(storage, value.to_string(), &false)
    }

    // Remove a value from the multi-map under the composite key (primary_key, secondary_key)
    pub fn remove(
        &self,
        storage: &mut dyn Storage,
        primary_key: &str,
        secondary_key: &str,
    ) {
        let k = self.create_submap_key(primary_key);
        let map: Map<String, bool> = Map::new_dyn(k);
        map.remove(storage, secondary_key.to_string());
    }

    // Get all values associated with a primary key by scanning the prefix
    pub fn is_empty(&self, storage: &dyn Storage, primary_key: &str) -> bool {
        let k = self.create_submap_key(primary_key);
        let map: Map<String, bool> = Map::new_dyn(k);
        map.is_empty(storage)
    }

}

impl<'a> MultiSet {
    // Function to get all keys associated with a primary key
    pub fn get_values<'c>(
        &'a self,
        storage: &'c dyn Storage,
        primary_key: &str,
        min: Option<Bound<'a, String>>,
        max: Option<Bound<'a, String>>,
        order: cosmwasm_std::Order,
    ) -> Box<dyn Iterator<Item = Result<String, cosmwasm_std::StdError>> + 'c> {
        let k = self.create_submap_key(primary_key);
        let map: Map<String, bool> = Map::new_dyn(k);
        map.keys(storage, min, max, order)
    }
}