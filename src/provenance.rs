use std::collections::BTreeSet;
use std::rc::Rc;
use crate::core_types::Address;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Provenance {
    pub deps: Option<Rc<BTreeSet<Address>>>,
}

impl Provenance {
    pub fn none() -> Self {
        Self { deps: None }
    }

    pub fn single(addr: Address) -> Self {
        let mut set = BTreeSet::new();
        set.insert(addr);
        Self { deps: Some(Rc::new(set)) }
    }

    pub fn merge(&self, other: &Self) -> Self {
        match (&self.deps, &other.deps) {
            (None, None) => Self::none(),
            (Some(d), None) => Self { deps: Some(d.clone()) },
            (None, Some(d)) => Self { deps: Some(d.clone()) },
            (Some(d1), Some(d2)) => {
                // Union
                let mut new_set = (**d1).clone();
                new_set.extend(d2.iter());
                Self { deps: Some(Rc::new(new_set)) }
            }
        }
    }
}
