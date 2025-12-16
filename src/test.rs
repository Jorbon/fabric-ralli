use crate::semantic_version::{SemanticVersion, SemanticVersionRange, simplify_range_set};


#[test]
fn version_cmp() {
    let test_versions = [
        "1.0.0-",
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
        "2.0.0",
        "2.1.0",
        "2.1.1",
    ];
    
    for i in 0..(test_versions.len() - 1) {
        if !(test_versions[i].parse::<SemanticVersion>().unwrap() < test_versions[i + 1].parse::<SemanticVersion>().unwrap()) {
            panic!("{}, {}", test_versions[i], test_versions[i + 1])
        }
    }
}

#[test]
fn version_merging() {
    for range in simplify_range_set(vec![
        ">=1.0 <1.2.0".parse().unwrap(),
        ">=1.2.0 <1.5.9".parse().unwrap(),
        ">=1.6.0 <1.5.9".parse().unwrap(),
    ]) {
        println!("{}", range);
    }
}
