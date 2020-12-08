use prost_build::Config;

fn main() {
    let mut config = Config::new();
    // config.compile_well_known_types();
    config.type_attribute("Sample", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(
        "Timestamp",
        "#[derive(serde::Serialize, serde::Deserialize)]",
    );
    config.field_attribute("Sample.id", "#[serde(rename = \"_id\", serialize_with = \"crate::db::id::serialize\", deserialize_with = \"crate::db::id::deserialize\")]");

    tonic_build::configure()
        .build_client(false)
        .compile_with_config(config, &["proto/sample.proto"], &["proto", "google"])
        .unwrap_or_else(|e| panic!("Failed to compile protos! {:?}", e));
}
