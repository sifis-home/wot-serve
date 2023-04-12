use serde::{Deserialize, Serialize};
use wot_td::{
    extend::{Extend, Extendable, ExtendableThing},
    hlist::*,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NilPlus<T> {
    #[serde(flatten)]
    pub field: T,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsPlus<T, U, V> {
    #[serde(flatten)]
    pub field: T,
    #[serde(flatten)]
    cons: Cons<U, V>,
}

impl<T, U, V> ConsPlus<T, U, V> {
    pub fn cons<Z>(self, value: Z) -> ConsPlus<T, Z, Cons<U, V>> {
        let Self { field, cons } = self;

        ConsPlus {
            field,
            cons: cons.cons(value),
        }
    }
}

impl<T> NilPlus<T> {
    pub fn cons<V>(self, value: V) -> ConsPlus<T, V, Nil> {
        let Self { field } = self;
        ConsPlus {
            field,
            cons: Nil::cons(value),
        }
    }
}

impl<T: Default> Extendable for NilPlus<T> {
    type Empty = NilPlus<T>;

    fn empty() -> Self {
        NilPlus {
            field: Default::default(),
        }
    }
}

impl<T: Default, U, V> Extendable for ConsPlus<T, U, V> {
    type Empty = NilPlus<T>;

    fn empty() -> Self::Empty {
        NilPlus {
            field: Default::default(),
        }
    }
}

impl<T, U> Extend<U> for NilPlus<T> {
    type Target = ConsPlus<T, U, Nil>;

    fn ext(self, u: U) -> Self::Target {
        let Self { field } = self;

        ConsPlus {
            field,
            cons: Nil::cons(u),
        }
    }
}

impl<T, U, V, Z> Extend<Z> for ConsPlus<T, U, V> {
    type Target = ConsPlus<T, Z, Cons<U, V>>;

    fn ext(self, t: Z) -> Self::Target {
        self.cons(t)
    }
}

impl<T: ExtendableThing> ExtendableThing for NilPlus<T> {
    type InteractionAffordance = NilPlus<T::InteractionAffordance>;
    type PropertyAffordance = NilPlus<T::PropertyAffordance>;
    type ActionAffordance = NilPlus<T::ActionAffordance>;
    type EventAffordance = NilPlus<T::EventAffordance>;
    type Form = NilPlus<T::Form>;
    type ExpectedResponse = NilPlus<T::ExpectedResponse>;
    type DataSchema = NilPlus<T::DataSchema>;
    type ObjectSchema = NilPlus<T::ObjectSchema>;
    type ArraySchema = NilPlus<T::ArraySchema>;
}

impl<T, U, V> ExtendableThing for ConsPlus<T, U, V>
where
    T: ExtendableThing,
    U: ExtendableThing,
    V: ExtendableThing,
{
    type InteractionAffordance =
        ConsPlus<T::InteractionAffordance, U::InteractionAffordance, V::InteractionAffordance>;
    type PropertyAffordance =
        ConsPlus<T::PropertyAffordance, U::PropertyAffordance, V::InteractionAffordance>;
    type ActionAffordance = ConsPlus<T::ActionAffordance, U::ActionAffordance, V::ActionAffordance>;
    type EventAffordance = ConsPlus<T::EventAffordance, U::EventAffordance, V::EventAffordance>;
    type Form = ConsPlus<T::Form, U::Form, V::Form>;
    type ExpectedResponse = ConsPlus<T::ExpectedResponse, U::ExpectedResponse, V::ExpectedResponse>;
    type DataSchema = ConsPlus<T::DataSchema, U::DataSchema, V::DataSchema>;
    type ObjectSchema = ConsPlus<T::ObjectSchema, U::ObjectSchema, V::ObjectSchema>;
    type ArraySchema = ConsPlus<T::ArraySchema, U::ArraySchema, U::ArraySchema>;
}

pub trait Holder<T> {
    fn field_ref(&self) -> &T;
    fn field_mut(&mut self) -> &mut T;
}

impl<T> Holder<T> for NilPlus<T> {
    fn field_ref(&self) -> &T {
        &self.field
    }

    fn field_mut(&mut self) -> &mut T {
        &mut self.field
    }
}

impl<T, U, V> Holder<T> for ConsPlus<T, U, V> {
    fn field_ref(&self) -> &T {
        &self.field
    }

    fn field_mut(&mut self) -> &mut T {
        &mut self.field
    }
}
