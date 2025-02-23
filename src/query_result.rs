//! same as <https://github.com/launchbadge/sqlx/blob/d4ae6ffd882ed2de1695c652888d809bc068554e/sqlx-sqlite/src/query_result.rs>

#[derive(Default)]
pub struct D1QueryResult {
    pub rows_affected: usize,
    pub last_insert_row_id: i64,
}

impl std::iter::Extend<Self> for D1QueryResult {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        for r in iter {
            self.rows_affected += r.rows_affected;
            self.last_insert_row_id = r.last_insert_row_id;
        }
    }
}
