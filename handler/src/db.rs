use std::{fmt::Display, num::NonZeroI64, ops::Deref};

use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Postgres};
use twilight_model::id::Id;

#[repr(transparent)]
// derive what we can from twilight_model::id::Id
#[derive(Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub(crate) struct DbId<T>(pub Id<T>);

impl<T> Deref for DbId<T> {
    type Target = Id<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Display for DbId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<T> Copy for DbId<T> {}

impl<T> Clone for DbId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for DbId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T> Eq for DbId<T> {}

impl<T> Encode<'_, Postgres> for DbId<T> {
    #[expect(
        clippy::unwrap_in_result,
        reason = "this should never occur, but we still wanna signal that"
    )]
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let val = NonZeroI64::new(self.0.get() as i64)
            .expect("twilight_model::id::Id is NonZero why are we here");

        Encode::<Postgres>::encode_by_ref(&val, buf)
    }
}

impl<T> Decode<'_, Postgres> for DbId<T> {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let decoded: NonZeroI64 = Decode::<Postgres>::decode(value)?;
        Ok(Self(Id::<T>::new(decoded.get() as u64)))
    }
}

impl<T> sqlx::Type<Postgres> for DbId<T> {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<Postgres>>::type_info()
    }
}

impl<T> From<i64> for DbId<T> {
    fn from(value: i64) -> Self {
        Self(Id::<T>::new(value as u64))
    }
}
impl<T> From<DbId<T>> for i64 {
    fn from(value: DbId<T>) -> Self {
        value.0.get() as Self
    }
}
