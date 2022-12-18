macro_rules! test_name { () => { "short_switches" } }

include!("glue/boilerplate.rs");

#[test]
fn no_switch() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(!cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 0);
    assert!(cfg.d.is_none());
}

#[test]
fn single_switch() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-a"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 0);
    assert!(cfg.d.is_none());
}

#[test]
fn two_switches() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-ab"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(cfg.b);
    assert_eq!(cfg.c, 0);
    assert!(cfg.d.is_none());
}

#[test]
fn three_switches() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-abc"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(cfg.b);
    assert_eq!(cfg.c, 1);
    assert!(cfg.d.is_none());
}

#[test]
fn two_separate_switches() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-a", "-b"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(cfg.b);
    assert_eq!(cfg.c, 0);
    assert!(cfg.d.is_none());
}

#[test]
fn two_groups() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-ab", "-c"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(cfg.b);
    assert_eq!(cfg.c, 1);
    assert!(cfg.d.is_none());
}

#[test]
fn value_separate() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-d", "42"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(!cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 0);
    assert_eq!(cfg.d, Some("42".to_owned()))
}

#[test]
fn value_together() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-d42"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(!cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 0);
    assert_eq!(cfg.d, Some("42".to_owned()))
}

#[test]
fn value_coalesced_together() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-ad42"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 0);
    assert_eq!(cfg.d, Some("42".to_owned()))
}

#[test]
fn value_coalesced_separate() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-ad", "42"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 0);
    assert_eq!(cfg.d, Some("42".to_owned()))
}

#[test]
fn value_coalesced_separate_count2() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-accd", "42"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 2);
    assert_eq!(cfg.d, Some("42".to_owned()))
}

#[test]
fn value_coalesced_separate_count3() {
    use std::iter;
    use std::path::PathBuf;

    let (cfg, mut tail, _) = config::Config::custom_args_and_optional_files(&["app", "-accd", "42", "-c"], iter::empty::<PathBuf>()).unwrap();
    assert!(tail.next().is_none());

    assert!(cfg.a);
    assert!(!cfg.b);
    assert_eq!(cfg.c, 3);
    assert_eq!(cfg.d, Some("42".to_owned()))
}
