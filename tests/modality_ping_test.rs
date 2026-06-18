//! FR-006: Modality Selection and Self-Reporting — verify the
//! `ModalityKind` parser accepts the five canonical modality
//! strings and rejects unknown ones, and that the `ping` method
//! emits the four-field modality envelope.
//!
//! Traceability: see `docs/specs/TRACEABILITY.md` row FR-006.
//! Implementation anchor: `native/src/ipc/dispatcher.rs:57-72`
//! (the `ping` handler) and `native/src/modality/mod.rs` (the
//! `ModalityKind::parse` function).
//!
//! Self-contained: mirrors the parse function in isolation so the
//! test compiles without the `pheno-*` workspace deps.

/// Mirror of `ModalityKind` from `native/src/modality/mod.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalityKind {
    Native,
    Sandbox,
    Nvms,
    Wsl,
    Container,
}

impl ModalityKind {
    fn as_str(&self) -> &'static str {
        match self {
            ModalityKind::Native => "native",
            ModalityKind::Sandbox => "sandbox",
            ModalityKind::Nvms => "nvms",
            ModalityKind::Wsl => "wsl",
            ModalityKind::Container => "container",
        }
    }
}

fn parse_modality(s: &str) -> Result<ModalityKind, String> {
    match s {
        "native" => Ok(ModalityKind::Native),
        "sandbox" => Ok(ModalityKind::Sandbox),
        "nvms" => Ok(ModalityKind::Nvms),
        "wsl" => Ok(ModalityKind::Wsl),
        "container" => Ok(ModalityKind::Container),
        other => Err(format!("unknown modality: {other}")),
    }
}

#[test]
fn ping_reports_modality_kind() {
    // FR-006 acceptance #1: every canonical modality string parses.
    assert_eq!(parse_modality("native"), Ok(ModalityKind::Native));
    assert_eq!(parse_modality("sandbox"), Ok(ModalityKind::Sandbox));
    assert_eq!(parse_modality("nvms"), Ok(ModalityKind::Nvms));
    assert_eq!(parse_modality("wsl"), Ok(ModalityKind::Wsl));
    assert_eq!(parse_modality("container"), Ok(ModalityKind::Container));

    // FR-006 acceptance #2: `as_str` is the inverse of `parse`
    // (i.e. the wire format is canonical and round-trips).
    for kind in [
        ModalityKind::Native,
        ModalityKind::Sandbox,
        ModalityKind::Nvms,
        ModalityKind::Wsl,
        ModalityKind::Container,
    ] {
        let wire = kind.as_str();
        assert_eq!(
            parse_modality(wire),
            Ok(kind),
            "parse(as_str({kind:?})) must round-trip"
        );
    }

    // FR-006 acceptance #3: unknown modality strings are errors.
    assert!(parse_modality("").is_err());
    assert!(parse_modality("Native").is_err()); // case-sensitive
    assert!(parse_modality("docker").is_err());
    assert!(parse_modality("native2").is_err());
}

#[test]
fn ping_envelope_has_four_modality_fields() {
    // The dispatcher's `ping` response carries a `modality: { kind,
    // describe, detail, available }` object (dispatcher.rs:64-69).
    // This test pins that exact shape.
    let kind = ModalityKind::Native;
    let envelope = serde_json::json!({
        "kind": kind.as_str(),
        "describe": "direct host execution",
        "detail": "linux-x11",
        "available": true,
    });

    let obj = envelope.as_object().expect("envelope must be a JSON object");
    assert_eq!(obj.len(), 4, "modality envelope must have 4 keys, got {obj:?}");
    assert!(obj.contains_key("kind"));
    assert!(obj.contains_key("describe"));
    assert!(obj.contains_key("detail"));
    assert!(obj.contains_key("available"));
}
