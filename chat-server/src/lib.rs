mod app;
mod config;

mod error;
mod models;

pub use app::*;
pub use config::AppConfig;
pub use error::AppError;

#[cfg(test)]
pub mod test {
    use anyhow::Result;
    use dotenvy::dotenv;
    use sqlx::{Pool, Postgres};
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    pub struct TestDb {
        _tdb: TestPg, // 保持 TestPg 活着
        pub pool: Pool<Postgres>,
    }

    // 这里如果直接返回pool会出现问题， tdb会被销毁， 连接就会被断开，所以需要返回一个struct 来保持 tdb 活着
    pub async fn create_test_pool() -> Result<TestDb> {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let tdb = TestPg::new(database_url, Path::new("../migrations"));
        let pool = tdb.get_pool().await;
        Ok(TestDb { _tdb: tdb, pool })
    }
}
