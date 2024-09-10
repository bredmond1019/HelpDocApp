use diesel::expression::AsExpression;
use diesel::sql_types::{ VarChar};
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql, Output};
use diesel::pg::{Pg, PgValue};
use serde::{Deserialize, Serialize};
use std::io::Write;


#[derive(Debug, Clone, PartialEq, AsExpression, Deserialize, Serialize)]
#[diesel(sql_type = VarChar)]
pub struct Vector(pub Vec<f32>);

impl FromSql<VarChar, Pg> for Vector {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let s = <String as FromSql<VarChar, Pg>>::from_sql(bytes)?;
        let values: Result<Vec<f32>, _> = s.trim_matches(|p| p == '[' || p == ']')
            .split(',')
            .map(|s| s.trim().parse::<f32>())
            .collect();
        Ok(Vector(values?))
    }
}

impl ToSql<VarChar, Pg> for Vector {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = format!("[{}]", self.0.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","));
        out.write_all(s.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}
