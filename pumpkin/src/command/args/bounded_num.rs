use core::f64;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use pumpkin_protocol::java::client::play::ArgumentType;
use pumpkin_util::text::TextComponent;

use crate::command::CommandSender;
use crate::command::args::ConsumeResult;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// Consumes a single generic num, but only if it's in bounds.
pub struct BoundedNumArgumentConsumer<T: ToFromNumber> {
    min_inclusive: Option<T>,
    max_inclusive: Option<T>,
    name: Option<&'static str>,
}

impl<T: ToFromNumber> ArgumentConsumer for BoundedNumArgumentConsumer<T>
where
    Self: GetClientSideArgParser,
{
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        // 1. Perform the synchronous mutable operation (pop) outside the Future.
        let s_opt: Option<&'a str> = args.pop();

        // 2. Process the entire argument and check bounds synchronously.
        let result: Option<Arg<'a>> = s_opt
            // Replace args.pop()?.parse::<T>().ok()?
            .and_then(|s| s.parse::<T>().ok())
            .map(|x| {
                // Check Upper Bound (max_inclusive)
                if let Some(max) = self.max_inclusive
                    && x > max
                {
                    return Arg::Num(Err(NotInBounds::UpperBound(x.to_number(), max.to_number())));
                }

                // Check Lower Bound (min_inclusive)
                if let Some(min) = self.min_inclusive
                    && x < min
                {
                    return Arg::Num(Err(NotInBounds::LowerBound(x.to_number(), min.to_number())));
                }

                // Success case
                Arg::Num(Ok(x.to_number()))
            });

        // 3. Return a Future that immediately resolves to the calculated result.
        Box::pin(async move { result })
    }
}

impl<'a, T: 'static + ToFromNumber> FindArg<'a> for BoundedNumArgumentConsumer<T> {
    type Data = Result<T, NotInBounds>;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Num(data)) => match data {
                Ok(num) => T::from_number(num).map_or_else(
                    || Err(CommandError::InvalidConsumption(Some(name.to_string()))),
                    |x| Ok(Ok(x)),
                ),
                Err(err) => Ok(Err(*err)),
            },
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NotInBounds {
    /// Number is lower than the lower bound
    LowerBound(Number, Number),
    /// Number is higher then the upper bound
    UpperBound(Number, Number),
}

impl From<NotInBounds> for CommandError {
    fn from(value: NotInBounds) -> Self {
        match value {
            NotInBounds::LowerBound(val, min) => Self::CommandFailed(TextComponent::text(format!(
                "{} must not be less than {}, found {}",
                val.qualifier(),
                min,
                val
            ))),
            NotInBounds::UpperBound(val, max) => Self::CommandFailed(TextComponent::text(format!(
                "{} must not be more than {}, found {}",
                val.qualifier(),
                max,
                val
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Number {
    F64(f64),
    F32(f32),
    I32(i32),
    #[allow(unused)]
    I64(i64),
}

impl Number {
    #[must_use]
    pub fn qualifier(&self) -> &'static str {
        match self {
            Self::F64(_) | Self::F32(_) => "Float",
            Self::I32(_) | Self::I64(_) => "Integer",
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F64(v) => write!(f, "{v:.2}"),
            Self::F32(v) => write!(f, "{v:.2}"),
            Self::I32(v) => write!(f, "{v}"),
            Self::I64(v) => write!(f, "{v}"),
        }
    }
}

impl<T: ToFromNumber> BoundedNumArgumentConsumer<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_inclusive: None,
            max_inclusive: None,
            name: None,
        }
    }

    #[must_use]
    pub const fn min(mut self, min_inclusive: T) -> Self {
        self.min_inclusive = Some(min_inclusive);
        self
    }

    #[must_use]
    #[allow(unused)]
    pub const fn max(mut self, max_inclusive: T) -> Self {
        self.max_inclusive = Some(max_inclusive);
        self
    }

    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }
}

pub trait ToFromNumber: PartialOrd + Copy + Send + Sync + FromStr {
    fn to_number(self) -> Number;
    fn from_number(arg: &Number) -> Option<Self>;
}

impl<T: ToFromNumber> Default for BoundedNumArgumentConsumer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl ToFromNumber for f64 {
    fn to_number(self) -> Number {
        Number::F64(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::F64(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<f64> {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Double {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::java::client::play::SuggestionProviders> {
        None
    }
}

impl ToFromNumber for f32 {
    fn to_number(self) -> Number {
        Number::F32(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::F32(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<f32> {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Float {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::java::client::play::SuggestionProviders> {
        None
    }
}

impl ToFromNumber for i32 {
    fn to_number(self) -> Number {
        Number::I32(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::I32(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<i32> {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Integer {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::java::client::play::SuggestionProviders> {
        None
    }
}

impl ToFromNumber for i64 {
    fn to_number(self) -> Number {
        Number::I64(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::I64(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<i64> {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Long {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::java::client::play::SuggestionProviders> {
        None
    }
}

impl<T: ToFromNumber> DefaultNameArgConsumer for BoundedNumArgumentConsumer<T>
where
    Self: ArgumentConsumer,
{
    fn default_name(&self) -> &'static str {
        // setting a single default name for all BoundedNumArgumentConsumer variants is probably a bad idea since it would lead to confusion
        self.name.expect("Only use *_default variants of methods with a BoundedNumArgumentConsumer that has a name.")
    }
}
