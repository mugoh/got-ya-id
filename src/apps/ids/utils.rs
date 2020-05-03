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

    pub fn serialize<S>(point: &Option<PgPoint>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(pt) = point {
            PgPointStruct {
                lat: pt.0,
                lon: pt.1,
            }
            .serialize(serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PgPoint>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let point: Option<PgPointStruct> = Option::deserialize(deserializer)?;
        if let Some(p) = point {
            Ok(Some(PgPoint(p.lat, p.lon)))
        } else {
            Ok(None)
        }
    }
}
