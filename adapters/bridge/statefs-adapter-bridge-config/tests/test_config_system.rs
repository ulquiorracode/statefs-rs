use statefs_adapter_bridge_config::{ConfigAccess, ConfigManager, ConfigMut, Defaults, DictSource};

#[test]
fn test_merge_vs_join_policy_semantics() {
    // 1. Initial baseline configuration
    let base = Defaults::new()
        .set("server/tickrate", 64)
        .set("server/hostname", "Initial Server")
        .set("server/max_players", 32);

    // 2. Incoming override configuration
    let incoming = Defaults::new()
        .set("server/tickrate", 128)
        .set("server/hostname", "Overridden Server")
        .set("server/map", "de_dust2");

    // Test MERGE: Incoming wins on conflicts
    let cfg_merge = ConfigManager::new()
        .join(base.clone())
        .merge(incoming.clone())
        .build()
        .unwrap();

    assert_eq!(cfg_merge.get::<i64>("server/tickrate").unwrap(), 128);
    assert_eq!(
        cfg_merge.get::<&str>("server/hostname").unwrap(),
        "Overridden Server"
    );
    assert_eq!(cfg_merge.get::<i64>("server/max_players").unwrap(), 32);
    assert_eq!(cfg_merge.get::<&str>("server/map").unwrap(), "de_dust2");

    // Test JOIN: Existing wins on conflicts (Incoming only fills gaps)
    let cfg_join = ConfigManager::new()
        .merge(base)
        .join(incoming)
        .build()
        .unwrap();

    assert_eq!(cfg_join.get::<i64>("server/tickrate").unwrap(), 64);
    assert_eq!(
        cfg_join.get::<&str>("server/hostname").unwrap(),
        "Initial Server"
    );
    assert_eq!(cfg_join.get::<i64>("server/max_players").unwrap(), 32);
    // 'server/map' was missing in base, so it gets filled in by join
    assert_eq!(cfg_join.get::<&str>("server/map").unwrap(), "de_dust2");
}

#[test]
fn test_unified_generic_get_and_zero_copy() {
    let cfg = ConfigManager::new()
        .merge(
            Defaults::new()
                .set("network/port", 27015)
                .set("network/bind_ip", "0.0.0.0")
                .set("network/rate", 100000)
                .set("network/loss_tolerant", true)
                .set("physics/gravity", 800.5),
        )
        .build()
        .unwrap();

    // Type inference
    let port: u16 = cfg.get("network/port").unwrap();
    assert_eq!(port, 27015);

    // Explicit turbofish
    let rate = cfg.get::<u32>("network/rate").unwrap();
    assert_eq!(rate, 100000);

    let tolerant = cfg.get::<bool>("network/loss_tolerant").unwrap();
    assert!(tolerant);

    let gravity = cfg.get::<f32>("physics/gravity").unwrap();
    assert_eq!(gravity, 800.5);

    // Zero-copy string slice borrowing directly from the trie
    let bind_ip: &str = cfg.get("network/bind_ip").unwrap();
    assert_eq!(bind_ip, "0.0.0.0");
}

#[test]
fn test_nested_builder_scopes() {
    let cfg = ConfigManager::builder()
        .join(Defaults::new().set("global/app_name", "GameHost"))
        // Scoped sub-tree for VIP plugin
        .scope("/plugins/vip", |vip| {
            vip.join(Defaults::new().set("flags", "a").set("cost", 50))
                .merge(
                    DictSource::new("dynamic_vip")
                        .insert("cost", 100)
                        .insert("active", true),
                )
        })
        // Scoped sub-tree for Admin plugin
        .scope("/plugins/admin", |admin| {
            admin.merge(Defaults::new().set("immunity", 10))
        })
        .build()
        .unwrap();

    assert_eq!(cfg.get::<&str>("global/app_name").unwrap(), "GameHost");
    assert_eq!(cfg.get::<&str>("plugins/vip/flags").unwrap(), "a");
    assert_eq!(cfg.get::<i64>("plugins/vip/cost").unwrap(), 100);
    assert!(cfg.get::<bool>("plugins/vip/active").unwrap());
    assert_eq!(cfg.get::<i64>("plugins/admin/immunity").unwrap(), 10);
}

#[test]
fn test_runtime_mutation_capabilities() {
    let mut cfg = ConfigManager::new()
        .merge(Defaults::new().set("game/timelimit", 20))
        .build()
        .unwrap();

    assert_eq!(cfg.get::<i64>("game/timelimit").unwrap(), 20);

    // ConfigMut capability in action
    cfg.set("game/timelimit", 30).unwrap();
    assert_eq!(cfg.get::<i64>("game/timelimit").unwrap(), 30);

    cfg.set("game/mapname", "de_inferno").unwrap();
    assert_eq!(cfg.get::<&str>("game/mapname").unwrap(), "de_inferno");

    assert!(cfg.remove("game/mapname").unwrap());
    assert!(!cfg.contains("game/mapname"));
}

#[cfg(feature = "toml")]
#[test]
fn test_toml_source_and_combinator_mounting() {
    use statefs_adapter_bridge_config::{ConfigSourcePort, Toml};

    let toml_str = r#"
[server]
port = 27015
hostname = "Mounted TOML"
"#;

    let cfg = ConfigManager::builder()
        .merge(Toml::string(toml_str).at("/mounted/service"))
        .build()
        .unwrap();

    assert_eq!(
        cfg.get::<u16>("mounted/service/server/port").unwrap(),
        27015
    );
    assert_eq!(
        cfg.get::<&str>("mounted/service/server/hostname").unwrap(),
        "Mounted TOML"
    );
}

#[cfg(feature = "json")]
#[test]
fn test_json_source_and_combinator_mounting() {
    use statefs_adapter_bridge_config::{ConfigSourcePort, Json};

    let json_str = r#"
{
    "features": {
        "radar": true,
        "grenade_trail": false
    }
}
"#;

    let cfg = ConfigManager::builder()
        .merge(Json::string(json_str).at("/plugins/visuals"))
        .build()
        .unwrap();

    assert!(cfg.get::<bool>("plugins/visuals/features/radar").unwrap());
    assert!(
        !cfg.get::<bool>("plugins/visuals/features/grenade_trail")
            .unwrap()
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_serde_typed_extraction() {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct ServerSettings {
        port: u16,
        hostname: String,
        max_players: u8,
    }

    let cfg = ConfigManager::builder()
        .merge(
            Defaults::new()
                .set("server/port", 27015)
                .set("server/hostname", "Dedicated Serde Server")
                .set("server/max_players", 32),
        )
        .build()
        .unwrap();

    let settings: ServerSettings = cfg.extract("server").unwrap();
    assert_eq!(
        settings,
        ServerSettings {
            port: 27015,
            hostname: "Dedicated Serde Server".to_string(),
            max_players: 32,
        }
    );
}
