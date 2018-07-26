extern crate weave_lib;

#[cfg(test)]
mod no_rules_tests {
    use std::collections::BTreeMap;
    use weave_lib::closet::*;
    use weave_lib::outfits::*;
    use weave_lib::outfits::Error::Validation;
    use weave_lib::outfits::ValidationError::MultipleItemsPerFamily;
    use weave_lib::outfits::ValidationError::UnknownItems;

    #[test]
    fn no_rules_no_selections() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);

        let expected = Ok(Outfit::new(vec![&jeans, &blue]));
        assert_eq!(
            expected,
            complete_outfit(closet, vec![])
        );
    }

    #[test]
    fn no_rules_one_selection() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);

        let expected = Ok(Outfit::new(vec![&jeans, &red]));
        assert_eq!(
            expected,
            complete_outfit(closet, vec![&red])
        );
    }

    #[test]
    fn no_rules_selection_for_each_family() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);

        let expected = Ok(Outfit::new(vec![&slacks, &blue]));
        assert_eq!(
            expected,
            complete_outfit(closet, vec![&slacks, &blue])
        );
    }

    #[test]
    fn no_rules_unknown_selection() {
        let blue = Item::new("blue");
        let red = Item::new("red");
        let black = Item::new("black");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);

        let expected = Err(Validation(UnknownItems(vec![&black])));
        assert_eq!(
            expected,
            complete_outfit(closet, vec![&jeans, &black])
        );
    }

    #[test]
    fn no_rules_more_selections_than_families() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);

        let expected = {
            let mut duplicates = BTreeMap::new();
            duplicates.insert(&pants, vec![&jeans, &slacks]);

            Err(Validation(MultipleItemsPerFamily(duplicates)))
        };

        assert_eq!(
            expected,
            complete_outfit(closet, vec![&jeans, &blue, &slacks])
        );
    }
}

#[cfg(test)]
mod exclusion_rules_tests {
    use weave_lib::closet::*;
    use weave_lib::outfits::*;
    use weave_lib::outfits::Error::Validation;
    use weave_lib::outfits::ValidationError::ConflictingItems;

    #[test]
    fn exclusion_rule_with_one_selection() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);
        let closet = closet.add_exclusion_rule(&blue, &jeans);

        let expected = Ok(Outfit::new(vec![&slacks, &blue]));
        assert_eq!(
            expected,
            complete_outfit(closet.clone(), vec![&blue])
        );

        let expected = Ok(Outfit::new(vec![&jeans, &red]));
        assert_eq!(
            expected,
            complete_outfit(closet.clone(), vec![&jeans])
        );
    }

    #[test]
    fn exclusion_rule_with_conflicting_selection() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);
        let closet = closet.add_exclusion_rule(&blue, &jeans);

        let expected = Err(Validation(ConflictingItems(vec![&jeans, &blue])));
        assert_eq!(
            expected,
            complete_outfit(closet, vec![&blue, &jeans])
        );
    }

    #[test]
    #[should_panic]
    fn exclusion_rules_with_impossible_selection() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);
        let closet = closet.add_exclusion_rule(&blue, &jeans);
        let closet = closet.add_exclusion_rule(&blue, &slacks);

        let expected = Ok(Outfit::new(vec![&blue]));
        assert_eq!(
            expected,
            complete_outfit(closet.clone(), vec![&blue])
        );
    }
}

#[cfg(test)]
mod inclusion_rules_tests {
    use weave_lib::closet::*;
    use weave_lib::outfits::*;

    #[test]
    fn inclusion_rule_with_one_selection() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);

        let expected = Ok(Outfit::new(vec![&jeans, &blue]));
        assert_eq!(
            expected,
            complete_outfit(closet.clone(), vec![])
        );


        let closet = closet.add_inclusion_rule(&jeans, &red);

        let expected = Ok(Outfit::new(vec![&jeans, &red]));
        assert_eq!(
            expected,
            complete_outfit(closet.clone(), vec![])
        );
    }

    #[test]
    fn inclusion_rule_is_one_way() {
        let blue = Item::new("blue");
        let red = Item::new("red");

        let jeans = Item::new("jeans");
        let slacks = Item::new("slacks");

        let shirts = Family::new("shirts");
        let pants = Family::new("pants");

        let closet = Closet::new();
        let closet = closet.add_item(&shirts, &blue);
        let closet = closet.add_item(&shirts, &red);
        let closet = closet.add_item(&pants, &jeans);
        let closet = closet.add_item(&pants, &slacks);
        let closet = closet.add_inclusion_rule(&red, &slacks);

        let expected = Ok(Outfit::new(vec![&slacks, &blue]));
        assert_eq!(
            expected,
            complete_outfit(closet.clone(), vec![&slacks])
        );
    }
}