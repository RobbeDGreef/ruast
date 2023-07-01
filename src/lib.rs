mod expr;
mod stmt;
mod token;
mod ty;

use std::fmt;
use std::ops::{Index, IndexMut};

pub use expr::*;
pub use stmt::*;
pub use token::*;
pub use ty::*;

#[macro_export]
macro_rules! impl_obvious_conversion {
    ($Enum: ident; $($Variant: ident $(,)?)*) => {
        $(
            impl From<$Variant> for $Enum {
                fn from(item: $Variant) -> Self {
                    Self::$Variant(item)
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_display_for_enum {
    ($Enum: ident; $($Variant: ident $(,)?)*) => {
        impl std::fmt::Display for $Enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$Variant(item) => write!(f, "{item}"),
                    )*
                }
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Crate {
    pub attrs: Vec<Attribute>,
    pub items: Vec<Item>,
}

impl Index<usize> for Crate {
    type Output = Item;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl IndexMut<usize> for Crate {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl fmt::Display for Crate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.items.iter() {
            writeln!(f, "{item}")?;
        }
        Ok(())
    }
}

impl Crate {
    pub fn new() -> Self {
        Self {
            attrs: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: impl Into<Item>) {
        self.items.push(item.into());
    }
}