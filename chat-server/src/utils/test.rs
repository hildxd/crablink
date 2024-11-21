#[cfg(test)]
pub mod utils {
    use anyhow::Result;
    use axum::{
        body::{Body, Bytes},
        response::Response,
    };
    use dotenvy::dotenv;
    use jwt_simple::reexports::serde_json;

    use http_body_util::BodyExt;
    use serde::Deserialize;
    use sqlx::{Pool, Postgres};
    use sqlx_db_tester::TestPg;
    use std::{ops::Deref, path::Path};

    pub struct TestDb {
        _tdb: TestPg, // 保持 TestPg 活着
        pub pool: Pool<Postgres>,
    }

    impl Deref for TestDb {
        type Target = Pool<Postgres>;

        fn deref(&self) -> &Self::Target {
            &self.pool
        }
    }

    // 这里如果直接返回pool会出现问题， tdb会被销毁， 连接就会被断开，所以需要返回一个struct 来保持 tdb 活着
    pub async fn create_test_pool() -> Result<TestDb> {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let tdb = TestPg::new(database_url, Path::new("../migrations"));
        let pool = tdb.get_pool().await;
        Ok(TestDb { _tdb: tdb, pool })
    }

    // for<'de> 就像是告诉编译器："别担心，这个类型可以处理任何生命周期的输入"
    pub async fn parser_response<T>(res: Response<Body>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>
    {
        let body = res.into_body().collect().await?.to_bytes();
        let ret = serde_json::from_slice::<T>(&body)?;
        Ok(ret)
    }
}
