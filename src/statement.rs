use std::{borrow::Cow, sync::Arc};

pub struct D1Statement<'q> {
    sql: Cow<'q, str>,
    n_params: usize,
    // columns: Arc<Vec<>>
}
