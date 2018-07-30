use bdd::closet::Closet;
use bdd::closet_builder::Error::ConflictingFamilies;
use bdd::closet_builder::Error::ExclusionError;
use bdd::closet_builder::Error::InclusionError;
use bdd::node::Node;
use bdd::node::Node::FalseLeaf;
use bdd::node::Node::TrueLeaf;
use core::Family;
use core::Item;
use std::collections::BTreeMap;
use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    ConflictingFamilies(Vec<(Item, Vec<Family>)>),
    InclusionError(Vec<(Family, Vec<Item>)>),
    ExclusionError(Vec<(Family, Vec<Item>)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosetBuilder {
    contents: BTreeMap<Family, Vec<Item>>,
    item_index: BTreeMap<Item, Family>,
    exclusions: BTreeMap<Item, Vec<Item>>,
    inclusions: BTreeMap<Item, Vec<Item>>,
}

impl ClosetBuilder {
    pub fn new() -> ClosetBuilder {
        ClosetBuilder {
            contents: BTreeMap::new(),
            item_index: BTreeMap::new(),
            exclusions: BTreeMap::new(),
            inclusions: BTreeMap::new(),
        }
    }

    pub fn add_item(mut self, family: &Family, item: &Item) -> ClosetBuilder {
        &self.contents.entry(family.clone())
            .or_insert(vec![])
            .push(item.clone());

        &self.item_index.entry(item.clone())
            .or_insert(family.clone());

        self
    }

    pub fn add_exclusion_rule(mut self, selection: &Item, exclusion: &Item) -> ClosetBuilder {
        &self.exclusions.entry(selection.clone())
            .or_insert(vec![])
            .push(exclusion.clone());
        &self.exclusions.entry(exclusion.clone())
            .or_insert(vec![])
            .push(selection.clone());

        self
    }

    pub fn add_inclusion_rule(mut self, selection: &Item, inclusion: &Item) -> ClosetBuilder {
        &self.inclusions.entry(selection.clone())
            .or_insert(vec![])
            .push(inclusion.clone());

        self
    }

    pub fn must_build(self) -> Closet {
        self.build().expect("expected build to return Closet")
    }

    pub fn build(&self) -> Result<Closet, Error> {
        self.validate()?;

        let root = self.contents.iter()
            .map(|(_, items)| items.iter().fold(FalseLeaf, |left_branch, item| Node::xor(item, left_branch)))
            .fold(TrueLeaf, |left_branch, family_node| left_branch & family_node);

        let item_index = self.item_index.clone();
        Ok(Closet::new(item_index, root))
    }

    fn validate(&self) -> Result<(), Error> {
        let conflicts = ClosetBuilder::find_conflicting_families(self);
        if !conflicts.is_empty() {
            return Err(ConflictingFamilies(conflicts));
        }

        let conflicts = ClosetBuilder::find_illegal_include_rules(self);
        if !conflicts.is_empty() {
            return Err(InclusionError(conflicts));
        }

        let conflicts = ClosetBuilder::find_illegal_exclude_rules(self);
        if !conflicts.is_empty() {
            return Err(ExclusionError(conflicts));
        }

        return Ok(());
    }

    fn find_conflicting_families(&self) -> Vec<(Item, Vec<Family>)> {
        self.contents.iter()
            .flat_map(|(family, items)| {
                items.iter()
                    .map(|item| {
                        let item_family = self.item_index
                            .get(item)
                            .expect(&format!("item `{:?}` does not have family", item));

                        if item_family != family {
                            Some((item.clone(), vec![item_family.clone(), family.clone()]))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|conflict| conflict.is_some())
            .map(|conflict| conflict.unwrap())
            .collect::<Vec<(Item, Vec<Family>)>>()
    }

    fn find_illegal_include_rules(&self) -> Vec<(Family, Vec<Item>)> {
        ClosetBuilder::find_illegal_rules(&self.inclusions, &self.item_index)
    }

    fn find_illegal_exclude_rules(&self) -> Vec<(Family, Vec<Item>)> {
        ClosetBuilder::find_illegal_rules(&self.exclusions, &self.item_index)
    }

    fn find_illegal_rules(rules: &BTreeMap<Item, Vec<Item>>, item_index: &BTreeMap<Item, Family>) -> Vec<(Family, Vec<Item>)> {
        let mut conflicts = rules.iter()
            .flat_map(|(selection, items)| {
                let selection_family = item_index
                    .get(selection)
                    .expect(&format!("item `{:?}` does not have family", selection));

                items.iter()
                    .map(|item| {
                        let item_family = item_index
                            .get(item)
                            .expect(&format!("item `{:?}` does not have family", item));

                        if selection_family == item_family {
                            let mut items = vec![selection.clone(), item.clone()];
                            items.sort();

                            Some((selection_family.clone(), items))
                        } else {
                            None
                        }
                    })
                    .collect::<HashSet<_>>()
            })
            .filter(|conflict| conflict.is_some())
            .map(|conflict| conflict.unwrap())
            .collect::<Vec<_>>();

        conflicts.dedup_by(|a, b| a.1 == b.1);
        conflicts
    }
}

#[cfg(test)]
mod tests {
    use bdd::node::Node;
    use bdd::node::Node::FalseLeaf;
    use bdd::node::Node::TrueLeaf;
    use core::Family;
    use core::Item;
    use super::ClosetBuilder;

    #[test]
    fn two_families_with_one_item_each() {
        let blue = Item::new("blue");
        let jeans = Item::new("jeans");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet_builder = ClosetBuilder::new()
            .add_item(&shirts, &blue)
            .add_item(&pants, &jeans);

        let closet = closet_builder.must_build();

        let expected_cousin_node = {
            let right_branch = Node::branch(&blue, FalseLeaf, TrueLeaf);
            let parent_branch = Node::branch(&jeans, FalseLeaf, right_branch);

            parent_branch
        };

        assert_eq!(
            &expected_cousin_node,
            closet.root()
        );


        let both_selected = {
            let root = closet.root();
            let root = Node::apply(root, &jeans, true);

            Node::apply(&root, &blue, true)
        };
        assert_eq!(
            TrueLeaf,
            both_selected
        );


        let blue_selected = {
            let root = closet.root();
            let root = Node::apply(root, &blue, true);

            Node::apply(&root, &jeans, false)
        };
        assert_eq!(
            FalseLeaf,
            blue_selected
        );


        let jeans_selected = {
            let root = closet.root();
            let root = Node::apply(root, &jeans, true);

            Node::apply(&root, &blue, false)
        };
        assert_eq!(
            FalseLeaf,
            jeans_selected
        );
    }

    #[test]
    fn one_families_with_two_items() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let shirts = Family::new("shirts");

        let closet_builder = ClosetBuilder::new()
            .add_item(&shirts, &red)
            .add_item(&shirts, &blue);

        let closet = closet_builder.must_build();

        let expected_sibling_node = {
            let left_branch = Node::branch(&red, FalseLeaf, TrueLeaf);
            let right_branch = Node::branch(&red, TrueLeaf, FalseLeaf);
            let parent_branch = Node::branch(&blue, left_branch.clone(), right_branch.clone());

            parent_branch
        };
        assert_eq!(
            &expected_sibling_node,
            closet.root()
        );


        let red_selected = {
            let root = closet.root();
            let root = Node::apply(root, &red, true);
            Node::apply(&root, &blue, false)
        };
        assert_eq!(
            TrueLeaf,
            red_selected
        );


        let blue_selected = {
            let root = closet.root();
            let root = Node::apply(root, &blue, true);
            Node::apply(&root, &red, false)
        };
        assert_eq!(
            TrueLeaf,
            blue_selected
        );


        let both_selected = {
            let root = closet.root();
            let root = Node::apply(root, &blue, true);
            Node::apply(&root, &red, true)
        };
        assert_eq!(
            FalseLeaf,
            both_selected
        );
    }
}
