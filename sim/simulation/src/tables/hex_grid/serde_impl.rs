use super::{HexGrid, TableRow};
use crate::prelude::Hexagon;
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;
use std::marker::PhantomData;

impl<Row> Serialize for HexGrid<Row>
where
    Row: TableRow + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("HexGrid", 2)?;
        state.serialize_field("radius", &self.bounds.radius)?;

        state.serialize_field(
            "values",
            &self
                .bounds
                .iter_points()
                .map(|p| unsafe { self.get_unchecked(p) })
                .collect::<Vec<_>>(),
        )?;

        state.end()
    }
}

struct HexGridVisitor<V>
where
    V: TableRow,
{
    _m: PhantomData<V>,
}

impl<'de, Row> Visitor<'de> for HexGridVisitor<Row>
where
    Row: TableRow + Deserialize<'de> + Default,
{
    type Value = HexGrid<Row>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "A `radius` field, containing an integer and a `values` field containing a list of Rows",
        )
    }

    fn visit_map<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        use std::borrow::Cow;

        let mut radius: Option<usize> = None;
        let mut values: Option<Vec<Row>> = None;
        while let Some(key) = seq.next_key::<Cow<String>>()? {
            match key.as_str() {
                "radius" => {
                    radius = seq.next_value()?;
                }
                "values" => {
                    values = seq.next_value()?;
                }
                _ => {}
            }
        }
        let radius = radius.ok_or_else(|| de::Error::missing_field("radius"))?;
        let values = values.ok_or_else(|| de::Error::missing_field("values"))?;

        let mut result = HexGrid::new(radius);

        let bounds = Hexagon::from_radius(radius as i32);

        let len = values.len();
        if bounds.area() != len {
            return Err(de::Error::custom(
                "More values were given than slots in the grid",
            ));
        }
        for (val, p) in values.into_iter().zip(bounds.iter_points()) {
            result.insert(p, val).map_err(|_| {
                de::Error::custom("Failed to insert value into HexGrid with given radius")
            })?;
        }

        Ok(result)
    }
}

impl<'de, Row> Deserialize<'de> for HexGrid<Row>
where
    Row: TableRow + Deserialize<'de> + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["radius", "values"];
        deserializer.deserialize_struct(
            "HexGrid",
            FIELDS,
            HexGridVisitor::<Row> {
                _m: Default::default(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::{prelude::SliceRandom, thread_rng, Rng};

    use super::*;

    #[test]
    fn test_de_serialize() {
        let mut rng = thread_rng();

        let mut grid = HexGrid::new(16);

        let grid_points = Hexagon::from_radius(16).iter_points().collect::<Vec<_>>();

        (0..128).for_each(|_| {
            let pos = *grid_points.as_slice().choose(&mut rng).unwrap();
            let val = rng.gen_range(0..128000000);
            grid.insert(pos, val).unwrap();
        });

        let s = serde_json::to_string(&grid).unwrap();
        dbg!(&s);
        let res: HexGrid<i32> = serde_json::from_str(s.as_str()).unwrap();

        assert_eq!(res.bounds(), grid.bounds());

        res.iter().zip(grid.iter()).for_each(|(act, exp)| {
            assert_eq!(exp, act);
        });
    }
}
