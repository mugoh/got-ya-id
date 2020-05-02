/// (De)Serializer for Point type
pub mod serde_pg_point {
    use diesel_geometry::data_types::PgPoint;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    /// Diesel Point type (de)serializer helper struct
    struct PgPointStruct {
        lat: f64,
        lon: f64,
    }

    pub fn serialize<S>(point: &PgPoint, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        PgPointStruct {
            lat: point.0,
            lon: point.1,
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PgPoint, D::Error>
    where
        D: Deserializer<'de>,
    {
        let point = PgPointStruct::deserialize(deserializer)?;
        Ok(PgPoint(point.lat, point.lon))
    }
}
