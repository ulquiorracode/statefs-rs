use statefs_adapter_bridge_config::{
    ConfigAccess, ConfigError, ConfigManager, ConfigMut, Defaults, Rule,
};

#[test]
fn test_validation_required_rule() {
    // Missing required key should fail at build time
    let res = ConfigManager::builder()
        .merge(Defaults::new().set("server/port", 27015))
        .rule(Rule::required("server/hostname"))
        .build();

    assert!(res.is_err());
    match res.unwrap_err() {
        ConfigError::Validation(report) => {
            assert_eq!(report.errors.len(), 1);
            assert_eq!(report.errors[0].path, "server/hostname");
            assert!(report.errors[0].message.contains("missing"));
        }
        err => panic!("Expected ConfigError::Validation, got: {err:?}"),
    }

    // When key is provided, build passes
    let valid_cfg = ConfigManager::builder()
        .merge(
            Defaults::new()
                .set("server/port", 27015)
                .set("server/hostname", "CS 1.6 Pro"),
        )
        .rule(Rule::required("server/hostname"))
        .build()
        .unwrap();

    assert_eq!(valid_cfg.get::<u16>("server/port").unwrap(), 27015);
}

#[test]
fn test_validation_range_rule() {
    // Out of bounds port (e.g. 70000)
    let res = ConfigManager::builder()
        .merge(Defaults::new().set("server/port", 70000))
        .rule(Rule::range("server/port", 1.0, 65535.0))
        .build();

    assert!(res.is_err());
    match res.unwrap_err() {
        ConfigError::Validation(report) => {
            assert_eq!(report.errors.len(), 1);
            assert_eq!(report.errors[0].path, "server/port");
            assert!(report.errors[0].message.contains("outside allowed range"));
        }
        err => panic!("Expected ConfigError::Validation, got: {err:?}"),
    }

    // Valid port within range
    let cfg = ConfigManager::builder()
        .merge(Defaults::new().set("server/port", 27015))
        .rule(Rule::range("server/port", 1.0, 65535.0))
        .build()
        .unwrap();

    assert_eq!(cfg.get::<u16>("server/port").unwrap(), 27015);
}

#[test]
fn test_validation_choices_rule() {
    // Invalid mode choice
    let res = ConfigManager::builder()
        .merge(Defaults::new().set("gameplay/mode", "battle_royale"))
        .rule(Rule::choices(
            "gameplay/mode",
            ["deathmatch", "classic", "gungame"],
        ))
        .build();

    assert!(res.is_err());
    match res.unwrap_err() {
        ConfigError::Validation(report) => {
            assert_eq!(report.errors.len(), 1);
            assert_eq!(report.errors[0].path, "gameplay/mode");
            assert!(report.errors[0].message.contains("not in allowed choices"));
        }
        err => panic!("Expected ConfigError::Validation, got: {err:?}"),
    }

    // Valid choice
    let cfg = ConfigManager::builder()
        .merge(Defaults::new().set("gameplay/mode", "deathmatch"))
        .rule(Rule::choices(
            "gameplay/mode",
            ["deathmatch", "classic", "gungame"],
        ))
        .build()
        .unwrap();

    assert_eq!(cfg.get::<&str>("gameplay/mode").unwrap(), "deathmatch");
}

#[test]
fn test_validation_custom_rule() {
    // Custom IP address validator
    let is_valid_ip = |val: Option<&statefs_core::Value>| {
        let Some(statefs_core::Value::String(s)) = val else {
            return Err("expected string IP".into());
        };
        if s.split('.').count() == 4 {
            Ok(())
        } else {
            Err("invalid IPv4 format".into())
        }
    };

    let res = ConfigManager::builder()
        .merge(Defaults::new().set("network/bind_ip", "not_an_ip"))
        .rule(Rule::custom("network/bind_ip", is_valid_ip))
        .build();

    assert!(res.is_err());

    let cfg = ConfigManager::builder()
        .merge(Defaults::new().set("network/bind_ip", "127.0.0.1"))
        .rule(Rule::custom("network/bind_ip", is_valid_ip))
        .build()
        .unwrap();

    assert_eq!(cfg.get::<&str>("network/bind_ip").unwrap(), "127.0.0.1");
}

#[test]
fn test_validation_runtime_mutation_enforcement() {
    let mut cfg = ConfigManager::builder()
        .merge(Defaults::new().set("server/tickrate", 64))
        .rule(Rule::range("server/tickrate", 30.0, 128.0))
        .build_validated()
        .unwrap();

    assert_eq!(cfg.get::<i64>("server/tickrate").unwrap(), 64);

    // Valid mutation
    cfg.set("server/tickrate", 128).unwrap();
    assert_eq!(cfg.get::<i64>("server/tickrate").unwrap(), 128);

    // Invalid mutation violating registered bounds
    let err = cfg.set("server/tickrate", 200);
    assert!(err.is_err());
    match err.unwrap_err() {
        ConfigError::Validation(report) => {
            assert_eq!(report.errors.len(), 1);
            assert_eq!(report.errors[0].path, "server/tickrate");
            assert!(report.errors[0].message.contains("outside allowed range"));
        }
        other => panic!("Expected validation error, got: {other:?}"),
    }
}
