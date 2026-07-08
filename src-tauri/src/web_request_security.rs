mod authority;
mod host;
mod same_origin;

pub(crate) use host::require_allowed_host;
pub(crate) use same_origin::require_same_origin_api_post;

#[cfg(test)]
mod tests;
