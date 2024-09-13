use super::*;

#[test]
fn test_tokenize_single_inheritance() {
    let content = String::from("class Test(Parent):");

    let actual = tokenize(content);

    let expected = vec![String::from("class"), String::from("Test(Parent):")];

    assert_eq!(actual, expected);
}

#[test]
fn test_tokenize_multiple_inheritance() {
    let content = String::from("class Test(Parent, Sibling):");

    let actual = tokenize(content);

    let expected = vec![
        String::from("class"),
        String::from("Test(Parent,"),
        String::from("Sibling):"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn test_get_pascal_case_no_class() {
    let tokens = vec![String::from("def"), String::from("func:")];

    let actual = get_pascal_case(&tokens);

    assert!(actual.is_empty());
}

#[test]
fn test_get_pascal_case_no_inheritance() {
    let tokens = vec![String::from("class"), String::from("Test:")];

    let actual = get_pascal_case(&tokens);

    let class = String::from("Test:");

    let expected: Vec<Vec<&String>> = vec![vec![&class]];

    assert_eq!(actual, expected);
}

#[test]
fn test_get_pascal_case_single_inheritance() {
    let tokens = vec![String::from("class"), String::from("Test(Parent):")];

    let actual = get_pascal_case(&tokens);

    let class = String::from("Test(Parent):");

    let expected: Vec<Vec<&String>> = vec![vec![&class]];

    assert_eq!(actual, expected);
}

#[test]
fn test_get_pascal_case_multiple_inheritance() {
    let tokens = vec![
        String::from("class"),
        String::from("Test(Parent,"),
        String::from("Sibling):"),
    ];

    let actual = get_pascal_case(&tokens);

    let class_1 = String::from("Test(Parent,");
    let class_2 = String::from("Sibling):");

    let expected: Vec<Vec<&String>> = vec![vec![&class_1, &class_2]];

    assert_eq!(actual, expected);
}

#[test]
fn test_get_child_classes_is_empty() {
    let class = String::from("Test:");

    let classes: Vec<Vec<&String>> = vec![vec![&class]];

    let actual = get_child_classes(classes);

    assert!(actual.is_empty());
}

#[test]
fn test_get_child_classes_single_inheritance() {
    let class = String::from("Test(Parent):");

    let classes: Vec<Vec<&String>> = vec![vec![&class]];

    let actual = get_child_classes(classes);

    let expected = vec![class];

    assert_eq!(actual, expected);
}

#[test]
fn test_get_child_classes_multiple_inheritance() {
    let class_1 = String::from("Test(Parent,");
    let class_2 = String::from("Sibling):");

    let classes: Vec<Vec<&String>> = vec![vec![&class_1, &class_2]];

    let actual = get_child_classes(classes);

    let expected = vec![String::from("Test(Parent, Sibling):")];

    assert_eq!(actual, expected);
}

#[test]
fn test_get_parent_class_single_inheritance() {
    let class = String::from("Test(Parent):");

    let actual = get_parent_class(&class);

    let expected = vec![String::from("Parent")];

    assert_eq!(actual, expected);
}

#[test]
fn test_get_parent_class_multiple_inheritance() {
    let class = String::from("Test(Parent, Sibling):");

    let actual = get_parent_class(&class);

    let expected = vec![String::from("Parent"), String::from("Sibling")];

    assert_eq!(actual, expected);
}
