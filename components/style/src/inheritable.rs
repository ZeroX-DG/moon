use super::value_processing::Property;
use std::collections::HashSet;

lazy_static! {
    pub static ref INHERITABLES: HashSet<Property> = {
        let mut set = HashSet::new();
        set.insert(Property::Color);
        set
    };
}
