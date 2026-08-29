//! Pagination. See `docs/04-dinero-fechas-y-tipos.md` §8.

use crate::error::{AppError, FieldError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    /// 1-based. There is no page zero.
    pub page: u32,
    /// `0` means "no paging": return everything.
    pub size: u32,
}

impl PageRequest {
    pub const DEFAULT_SIZE: u32 = 30;
    pub const ALLOWED_SIZES: [u32; 5] = [10, 30, 50, 100, 0];

    #[must_use]
    pub const fn new(page: u32, size: u32) -> Self {
        Self { page, size }
    }

    #[must_use]
    pub const fn offset(self) -> u64 {
        (self.page.saturating_sub(1)) as u64 * self.size as u64
    }

    /// `None` when unpaged, so the caller emits no `LIMIT` at all.
    #[must_use]
    pub const fn limit(self) -> Option<u64> {
        if self.size == 0 {
            None
        } else {
            Some(self.size as u64)
        }
    }

    /// Rejects a size outside the allowed list so nobody can ask for a million rows, and a page
    /// below 1 so `offset` stays meaningful.
    pub fn validate(self) -> Result<(), AppError> {
        let mut errors = Vec::new();
        if !Self::ALLOWED_SIZES.contains(&self.size) {
            errors.push(
                FieldError::new("size", "Validation.Paging.SizeNotAllowed").with_param(
                    "allowed",
                    Self::ALLOWED_SIZES.map(|s| s.to_string()).join(", "),
                ),
            );
        }
        if self.page < 1 {
            errors.push(FieldError::new("page", "Validation.Paging.PageOutOfRange"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(errors))
        }
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            size: Self::DEFAULT_SIZE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub size: u32,
    pub total_pages: u32,
    pub has_previous: bool,
    pub has_next: bool,
}

impl<T> PagedResult<T> {
    /// Derives every count from the request and the total, so no caller computes them by hand and
    /// gets `has_next` wrong on the last page.
    #[must_use]
    pub fn new(items: Vec<T>, total_count: u64, request: PageRequest) -> Self {
        let total_pages = if request.size == 0 {
            u32::from(total_count > 0)
        } else {
            total_count.div_ceil(u64::from(request.size)) as u32
        };
        Self {
            items,
            total_count,
            page: request.page,
            size: request.size,
            total_pages,
            has_previous: request.page > 1,
            has_next: request.page < total_pages,
        }
    }

    #[must_use]
    pub fn empty(request: PageRequest) -> Self {
        Self::new(Vec::new(), 0, request)
    }

    /// Maps the items while preserving every count.
    #[must_use]
    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> PagedResult<U> {
        PagedResult {
            items: self.items.into_iter().map(f).collect(),
            total_count: self.total_count,
            page: self.page,
            size: self.size,
            total_pages: self.total_pages,
            has_previous: self.has_previous,
            has_next: self.has_next,
        }
    }

    /// Maps the items with a fallible function, e.g. a mapper that can hit a domain error.
    pub fn try_map<U, E, F: FnMut(T) -> Result<U, E>>(self, f: F) -> Result<PagedResult<U>, E> {
        Ok(PagedResult {
            items: self.items.into_iter().map(f).collect::<Result<_, E>>()?,
            total_count: self.total_count,
            page: self.page,
            size: self.size,
            total_pages: self.total_pages,
            has_previous: self.has_previous,
            has_next: self.has_next,
        })
    }
}
