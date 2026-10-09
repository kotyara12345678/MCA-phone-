use thiserror::Error;

/// Input validation failures. Everything arriving from a client, a provider or
/// an LLM response is funnelled through this before it reaches the domain.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    #[error("field `{0}` must not be empty")]
    Empty(&'static str),
    #[error("field `{field}` is too long (max {max} chars, got {actual})")]
    TooLong {
        field: &'static str,
        max: usize,
        actual: usize,
    },
    #[error("field `{field}` exceeds {max} bytes")]
    TooLarge { field: &'static str, max: usize },
    #[error("field `{field}` is not a valid value: {reason}")]
    Invalid { field: &'static str, reason: String },
    #[error("value `{value}` is out of range for `{field}` ({min}..={max})")]
    OutOfRange {
        field: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },
    #[error("text is not valid UTF-8 / contains control characters")]
    ControlCharacters,
    #[error("{0}")]
    Custom(String),
}

impl ValidationError {
    pub fn empty(field: &'static str) -> Self {
        Self::Empty(field)
    }

    pub fn invalid(field: &'static str, reason: impl Into<String>) -> Self {
        Self::Invalid {
            field,
            reason: reason.into(),
        }
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

pub type Validated<T> = Result<T, ValidationError>;

/// Normalises an incoming free-text field, collapsing whitespace and trimming.
pub fn clean_text(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Rejects control characters (except newline/tab) that could corrupt logs.
pub fn has_control_characters(raw: &str) -> bool {
    raw.chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
}

/// Enforces a maximum character length after normalisation.
pub fn check_len(field: &'static str, value: &str, max: usize) -> Validated<()> {
    if has_control_characters(value) {
        return Err(ValidationError::ControlCharacters);
    }
    let len = value.chars().count();
    if len > max {
        return Err(ValidationError::TooLong {
            field,
            max,
            actual: len,
        });
    }
    Ok(())
}

/// Validates a physical quantity with sane transport bounds.
pub fn check_quantity(field: &'static str, value: f64, min: f64, max: f64) -> Validated<f64> {
    if !value.is_finite() {
        return Err(ValidationError::invalid(field, "not a finite number"));
    }
    if value < min || value > max {
        return Err(ValidationError::OutOfRange {
            field,
            value,
            min,
            max,
        });
    }
    Ok(value)
}

#[cfg(test)]
mod tests;
