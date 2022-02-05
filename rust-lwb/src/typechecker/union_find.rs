use crate::typechecker::constraints::Variable;
use crate::typechecker::error::{GeneratedTypeError, TypeError};
use crate::typechecker::Type;
use itertools::Itertools;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

struct ById<'var, TYPE: Type>(&'var Variable<TYPE>);

impl<'var, TYPE: Type> Eq for ById<'var, TYPE> {}

impl<'var, TYPE: Type> PartialEq for ById<'var, TYPE> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id().eq(&other.0.id())
    }
}

impl<'var, TYPE: Type> Hash for ById<'var, TYPE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.id().hash(state)
    }
}

struct Element<'var, TYPE: Type> {
    pub parent: Cell<&'var Variable<TYPE>>,
    // pub original_certainty: Cell<usize>,
    // pub set_certainty: Cell<usize>,
    pub depth: Cell<usize>,
}

pub struct UnionFind<'var, TYPE: Type> {
    ds: HashMap<ById<'var, TYPE>, Element<'var, TYPE>>,
}

impl<'var, TYPE: Type> Debug for UnionFind<'var, TYPE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.ds {
            let (repr, elem, _depth) = self.find_internal(key.0);
            writeln!(f,
                "{: <20} {: <20} ({:?}) --> {: <20} ({})",
                key.0.dbg_msg(),
                format!("{:?}", key.0),
                value.depth.get(),
                format!("{:?}", repr),
                elem.depth.get()
            )?;
        }

        Ok(())
        // write!(
        //     f,
        //     "[{}]",
        //     self.ds
        //         .iter()
        //         .map(|(key, value)| { format!("{:?}->{:?} ({})", key.0, self.find_representative(key.0), value.certainty.get()) })
        //         .join(", ")
        // )
    }
}

impl<'var, TYPE: Type> UnionFind<'var, TYPE> {
    pub fn new() -> Self {
        Self {
            ds: Default::default(),
        }
    }

    pub fn insert(&mut self, var: &'var Variable<TYPE>) {
        self.ds.insert(
            ById(var),
            Element {
                parent: Cell::new(var),
                // set_certainty: certainty.clone(),
                // original_certainty: certainty
                depth: Cell::new(1)
            },
        );
    }

    fn find_internal(
        &self,
        var: &'var Variable<TYPE>,
    ) -> (&'var Variable<TYPE>, &Element<'var, TYPE>, usize) {
        // todo: path splitting/halving?
        let el = self
            .ds
            .get(&ById(var))
            .expect("variable not found in union find. Use insert on all variables first.");
        if ById(el.parent.get()) != ById(var) {
            let (var, elem, depth) = self.find_internal(el.parent.get());
            el.parent.set(var);
            el.depth.set(depth);
            (var, elem, depth + 1)
        } else {
            (var, el, 1)
        }
    }

    pub fn find_representative(&self, var: &'var Variable<TYPE>) -> &'var Variable<TYPE> {
        self.find_internal(var).0
    }

    pub fn union(
        &self,
        a: &'var Variable<TYPE>,
        b: &'var Variable<TYPE>,
    ) -> Result<bool, TypeError<TYPE>> {
        // union two variables. First find the root of both a and b (ar and br). ar_data and br_data
        // contain information about the previous parent of ar/br. These will be updated.
        // When a known variable unions with an unknown variable, the parent becomes the known variable.
        // Two known variables simply can't union since that would mean some value in the ast has two
        // types. TODO: allow users to define what happens in these cases (by passing a closure?)
        // When two unknown (free) variables are unioned, that's just fine.
        let (ar, ar_data, _a_depth) = self.find_internal(a);
        let (br, br_data, _b_depth) = self.find_internal(b);

        if ById(ar) == ById(br) {
            return Ok(false);
        }

        match (ar, br) {
            (Variable::Known(a), Variable::Known(b)) if a.value != b.value => {
                return Err(GeneratedTypeError::CantUnify(
                    a.span.borrow().clone(),
                    a.clone(),
                    ar_data.depth.get() < br_data.depth.get(),
                    b.span.borrow().clone(),
                    b.clone(),
                    ar_data.depth.get() > br_data.depth.get(),
                )
                .into());
            }
            (va @ Variable::Known(_), vb @ Variable::Known(_)) => {
                br_data.parent.set(va);
                ar_data.depth.set(ar_data.depth.get() + 1);

                // ar_data.certainty.set(ar_data.certainty.get() + 1);
                va.merge_span(vb)
            }
            (va @ Variable::Free(_), vb @ Variable::Known(_)) => {
                ar_data.parent.set(vb);
                br_data.depth.set(br_data.depth.get() + 1);
                va.merge_span(vb)
            }
            (va @ Variable::Known(_), vb @ Variable::Free(_)) => {
                br_data.parent.set(va);
                ar_data.depth.set(ar_data.depth.get() + 1);
                va.merge_span(vb)
            }
            (va @ Variable::Free(_), vb @ Variable::Free(_)) => {
                br_data.parent.set(va);
                ar_data.depth.set(ar_data.depth.get() + 1);
                va.merge_span(vb)
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::typechecker::constraints::Variable;
    use crate::typechecker::union_find::{ById, UnionFind};
    use crate::typechecker::Type;
    use std::collections::HashSet;

    #[derive(Debug, PartialEq)]
    enum TestType {
        A,
        B,
    }

    impl Type for TestType {}

    #[test]
    fn test_init() {
        let mut uf = UnionFind::new();
        let v0 = Variable::<TestType>::new_free(None);
        let v1 = Variable::<TestType>::new_known(TestType::A, None);
        let v2 = Variable::<TestType>::new_known(TestType::B, None);

        let vars = vec![v0, v1, v2];

        for var in &vars {
            uf.insert(var);
        }

        // initially everything maps to itself
        for i in &vars {
            assert_eq!(uf.find_representative(i).id(), i.id())
        }
    }

    #[test]
    fn test_union() {
        let mut uf = UnionFind::new();
        let v0 = Variable::<TestType>::new_free(None);
        let v1 = Variable::<TestType>::new_free(None);
        let v2 = Variable::<TestType>::new_free(None);
        let v3 = Variable::<TestType>::new_free(None);
        let v4 = Variable::<TestType>::new_known(TestType::A, None);
        let v5 = Variable::<TestType>::new_known(TestType::B, None);

        let vars = vec![&v0, &v1, &v2, &v3, &v4, &v5];

        for var in vars {
            uf.insert(var);
        }

        // union free + free
        assert!(uf.union(&v0, &v1).is_ok());
        // union known + free
        assert!(uf.union(&v4, &v2).is_ok());
        // union free + known
        assert!(uf.union(&v3, &v5).is_ok());

        // union known + known
        assert!(uf.union(&v4, &v5).is_err());
    }

    #[test]
    fn balanced_free_union() {
        let mut uf = UnionFind::new();
        let v0 = Variable::<TestType>::new_free(None);
        let v1 = Variable::<TestType>::new_free(None);
        let v2 = Variable::<TestType>::new_free(None);
        let v3 = Variable::<TestType>::new_free(None);
        let v4 = Variable::<TestType>::new_free(None);
        let v5 = Variable::<TestType>::new_free(None);
        let v6 = Variable::<TestType>::new_free(None);
        let v7 = Variable::<TestType>::new_free(None);

        let vars = vec![&v0, &v1, &v2, &v3, &v4, &v5, &v6, &v7];

        for &var in &vars {
            uf.insert(var);
        }

        assert!(uf.union(&v0, &v1).is_ok());
        assert!(uf.union(&v0, &v2).is_ok());
        assert!(uf.union(&v0, &v3).is_ok());
        assert!(uf.union(&v0, &v4).is_ok());
        assert!(uf.union(&v5, &v6).is_ok());
        assert!(uf.union(&v7, &v6).is_ok());
        assert!(uf.union(&v5, &v2).is_ok());

        let mut set = HashSet::new();
        for &i in &vars {
            let repr = uf.find_representative(i);
            set.insert(ById(repr));
        }

        assert_eq!(set.len(), 1);
    }
}
