use nve_core::domain::version::{matches_semver, ParsedVersion};

#[test]
fn parse_valido_major() {
    let v = ParsedVersion::parse("18").expect("ok");
    assert_eq!(v.major, 18);
    assert_eq!(v.minor, None);
    assert_eq!(v.patch, None);
    assert_eq!(v.full_version, "18");
}

#[test]
fn parse_valido_major_minor() {
    let v = ParsedVersion::parse("18.19").expect("ok");
    assert_eq!(v.major, 18);
    assert_eq!(v.minor, Some(19));
    assert_eq!(v.patch, None);
    assert_eq!(v.full_version, "18.19");
}

#[test]
fn parse_valido_major_minor_patch() {
    let v = ParsedVersion::parse("18.19.1").expect("ok");
    assert_eq!(v.major, 18);
    assert_eq!(v.minor, Some(19));
    assert_eq!(v.patch, Some(1));
    assert_eq!(v.full_version, "18.19.1");
}

#[test]
fn parse_invalido_varios() {
    for s in ["", ".", "a", "1.", "1.a", "1.2.", "1.2.a", "1.2.3.4"] {
        assert!(ParsedVersion::parse(s).is_err(), "debería fallar: {s}");
    }
}

#[test]
fn matches_major_solo_cubre_todas_las_menores_y_patches() {
    let spec = ParsedVersion::parse("18").unwrap();
    assert!(matches_semver("18.0.0", &spec));
    assert!(matches_semver("18.19.1", &spec));
    assert!(matches_semver("18.99.99", &spec));
    assert!(!matches_semver("17.9.9", &spec));
    assert!(!matches_semver("19.0.0", &spec));
}

#[test]
fn matches_major_minor_cubre_todos_los_patches_de_esa_menor() {
    let spec = ParsedVersion::parse("20.3").unwrap();
    assert!(matches_semver("20.3.0", &spec));
    assert!(matches_semver("20.3.5", &spec));
    assert!(!matches_semver("20.4.0", &spec));
    assert!(!matches_semver("19.3.0", &spec));
}

#[test]
fn matches_exact_patch_exige_coincidir_los_tres() {
    let spec = ParsedVersion::parse("22.1.7").unwrap();
    assert!(matches_semver("22.1.7", &spec));
    assert!(!matches_semver("22.1.6", &spec));
    assert!(!matches_semver("22.2.7", &spec));
    assert!(!matches_semver("21.1.7", &spec));
}

#[test]
fn matches_acepta_prereleases_y_build_metadata_si_numeros_coinciden() {
    let spec = ParsedVersion::parse("18.19.1").unwrap();
    assert!(matches_semver("18.19.1-beta.1", &spec));
    assert!(matches_semver("18.19.1+build.42", &spec));
    assert!(matches_semver("18.19.1-beta.1+exp.sha", &spec));
}

#[test]
fn matches_devuelve_false_si_ver_no_es_semver_valido() {
    let spec = ParsedVersion::parse("18").unwrap();
    for s in ["", "18.x", "1.2.3.4", "foo", "v18"] {
        assert!(!matches_semver(s, &spec), "no debería machar: {s}");
    }
}
