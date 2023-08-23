// Copyright 2022 Zinc Labs Inc. and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use async_trait::async_trait;
use chrono::Utc;
use once_cell::sync::Lazy;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Pool, Postgres, QueryBuilder, Row,
};
use std::str::FromStr;

use crate::common::infra::{
    config::CONFIG,
    errors::{Error, Result},
};
use crate::common::meta::{
    common::{FileKey, FileMeta},
    stream::PartitionTimeLevel,
    StreamType,
};

static CLIENT: Lazy<Pool<Postgres>> = Lazy::new(connect);

fn connect() -> Pool<Postgres> {
    let db_opts = PgConnectOptions::from_str(&CONFIG.common.file_list_postgres_dsn)
        .expect("postgres connect options create failed")
        .disable_statement_logging();

    let pool_opts = PgPoolOptions::new();
    let pool_opts = pool_opts.min_connections(CONFIG.limit.cpu_num as u32);
    let pool_opts = pool_opts.max_connections(CONFIG.limit.query_thread_num as u32);
    pool_opts.connect_lazy_with(db_opts)
}

pub struct PostgresFileList {}

impl PostgresFileList {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PostgresFileList {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::FileList for PostgresFileList {
    async fn add(&self, file: &str, meta: &FileMeta) -> Result<()> {
        let pool = CLIENT.clone();
        let (stream_key, date_key, file_name) = super::parse_file_key_columns(file)?;
        match  sqlx::query(
            r#"
INSERT INTO file_list (stream, date, file, deleted, min_ts, max_ts, records, original_size, compressed_size)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);
        "#,
        )
        .bind(stream_key)
        .bind(date_key)
        .bind(file_name)
        .bind(false)
        .bind(meta.min_ts)
        .bind(meta.max_ts)
        .bind(meta.records as i64)
        .bind(meta.original_size as i64)
        .bind(meta.compressed_size as i64)
        .execute(&pool)
        .await {
            Err(sqlx::Error::Database(e)) => if e.is_unique_violation() {
                  Ok(())
            } else {
                  Err(Error::Message(e.to_string()))
            },
            Err(e) =>  Err(e.into()),
            Ok(_) => Ok(()),
        }
    }

    async fn remove(&self, file: &str) -> Result<()> {
        let pool = CLIENT.clone();
        let (stream_key, date_key, file_name) = super::parse_file_key_columns(file)?;
        sqlx::query(
            r#"
DELETE FROM file_list 
    WHERE stream = $1 AND date = $2 AND file = $3;"#,
        )
        .bind(stream_key)
        .bind(date_key)
        .bind(file_name)
        .execute(&pool)
        .await?;
        Ok(())
    }

    async fn batch_add(&self, files: &[FileKey]) -> Result<()> {
        let pool = CLIENT.clone();
        let chunks = files.chunks(100);
        for files in chunks {
            let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO file_list (stream, date, file, deleted, min_ts, max_ts, records, original_size, compressed_size)");
            query_builder.push_values(files, |mut b, item| {
                let (stream_key, date_key, file_name) =
                    super::parse_file_key_columns(&item.key).expect("parse file key failed");
                b.push_bind(stream_key)
                    .push_bind(date_key)
                    .push_bind(file_name)
                    .push_bind(false)
                    .push_bind(item.meta.min_ts)
                    .push_bind(item.meta.max_ts)
                    .push_bind(item.meta.records as i64)
                    .push_bind(item.meta.original_size as i64)
                    .push_bind(item.meta.compressed_size as i64);
            });
            match query_builder.build().execute(&pool).await {
                Ok(_) => {}
                Err(sqlx::Error::Database(e)) => {
                    if e.is_unique_violation() {
                        // batch insert got unique error, convert to single insert
                        for file in files {
                            self.add(&file.key, &file.meta).await?;
                        }
                    } else {
                        return Err(Error::Message(e.to_string()));
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    async fn batch_remove(&self, files: &[String]) -> Result<()> {
        let pool = CLIENT.clone();
        let chunks = files.chunks(100);
        for files in chunks {
            let mut tx = pool.begin().await?;
            for file in files {
                let (stream_key, date_key, file_name) = super::parse_file_key_columns(file)?;
                let sql = format!("DELETE FROM file_list WHERE stream = '{stream_key}' AND date = '{date_key}' AND file = '{file_name}';");
                match sqlx::query(&sql).execute(&mut *tx).await {
                    Ok(_) => {}
                    Err(e) => return Err(e.into()),
                }
            }
            tx.commit().await?;
        }
        Ok(())
    }

    async fn get(&self, file: &str) -> Result<FileMeta> {
        let pool = CLIENT.clone();
        let (stream_key, date_key, file_name) = super::parse_file_key_columns(file)?;
        let ret = sqlx::query_as::<_, FileRecord>(
            r#"
SELECT stream, date, file, deleted, min_ts, max_ts, records, original_size, compressed_size
    FROM file_list WHERE stream = $1 AND date = $2 AND file = $3;
        "#,
        )
        .bind(stream_key)
        .bind(date_key)
        .bind(file_name)
        .fetch_one(&pool)
        .await?;
        Ok(FileMeta::from(&ret))
    }

    async fn list(&self) -> Result<Vec<(String, FileMeta)>> {
        let pool = CLIENT.clone();
        let ret = sqlx::query_as::<_, FileRecord>(
            r#"
SELECT stream, date, file, deleted, min_ts, max_ts, records, original_size, compressed_size
    FROM file_list;
        "#,
        )
        .fetch_all(&pool)
        .await?;
        Ok(ret
            .into_iter()
            .map(|r| {
                (
                    format!("files/{}/{}/{}", r.stream, r.date, r.file),
                    FileMeta::from(&r),
                )
            })
            .collect())
    }

    async fn query(
        &self,
        org_id: &str,
        stream_type: StreamType,
        stream_name: &str,
        _time_level: PartitionTimeLevel,
        time_range: (i64, i64),
    ) -> Result<Vec<(String, FileMeta)>> {
        let (time_start, mut time_end) = time_range;
        if time_start == 0 {
            return Err(Error::Message(
                "Disallow empty time range query".to_string(),
            ));
        }
        if time_end == 0 {
            time_end = Utc::now().timestamp_micros();
        }

        let stream_key = format!("{org_id}/{stream_type}/{stream_name}");

        let pool = CLIENT.clone();
        let ret = sqlx::query_as::<_, FileRecord>(
            r#"
SELECT stream, date, file, deleted, min_ts, max_ts, records, original_size, compressed_size
    FROM file_list 
    WHERE stream = $1 AND min_ts >= $2 AND max_ts <= $3;
        "#,
        )
        .bind(stream_key)
        .bind(time_start)
        .bind(time_end)
        .fetch_all(&pool)
        .await?;
        Ok(ret
            .into_iter()
            .map(|r| {
                (
                    format!("files/{}/{}/{}", r.stream, r.date, r.file),
                    FileMeta::from(&r),
                )
            })
            .collect())
    }

    async fn contains(&self, file: &str) -> Result<bool> {
        let pool = CLIENT.clone();
        let (stream_key, date_key, file_name) = super::parse_file_key_columns(file)?;
        let ret = sqlx::query(
            r#"
SELECT stream, date, file, deleted, min_ts, max_ts, records, original_size, compressed_size
    FROM file_list WHERE stream = $1 AND date = $2 AND file = $3;
        "#,
        )
        .bind(stream_key)
        .bind(date_key)
        .bind(file_name)
        .fetch_one(&pool)
        .await;
        if let Err(sqlx::Error::RowNotFound) = ret {
            return Ok(false);
        }
        Ok(!ret.unwrap().is_empty())
    }

    async fn len(&self) -> usize {
        let pool = CLIENT.clone();
        let ret = match sqlx::query(r#"SELECT COUNT(*) as num FROM file_list;"#)
            .fetch_one(&pool)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::error!("[POSTGRES] get file list len error: {}", e);
                return 0;
            }
        };
        match ret.try_get::<i64, &str>("num") {
            Ok(v) => v as usize,
            _ => 0,
        }
    }

    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    async fn clear(&self) -> Result<()> {
        let pool = CLIENT.clone();
        sqlx::query(r#"DELETE FROM file_list;"#)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct FileRecord {
    pub stream: String,
    pub date: String,
    pub file: String,
    pub deleted: bool,
    pub min_ts: i64,
    pub max_ts: i64,
    pub records: i64,
    pub original_size: i64,
    pub compressed_size: i64,
}

impl From<&FileRecord> for FileMeta {
    fn from(record: &FileRecord) -> Self {
        Self {
            min_ts: record.min_ts,
            max_ts: record.max_ts,
            records: record.records as u64,
            original_size: record.original_size as u64,
            compressed_size: record.compressed_size as u64,
        }
    }
}

pub async fn create_table() -> Result<()> {
    let pool = CLIENT.clone();
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS file_list
(
    id      INT GENERATED ALWAYS AS IDENTITY,
    stream  VARCHAR not null,
    date    VARCHAR not null,
    file    VARCHAR not null,
    deleted BOOLEAN default false not null,
    min_ts  BIGINT not null,
    max_ts  BIGINT not null,
    records BIGINT not null,
    original_size   BIGINT not null,
    compressed_size BIGINT not null
);
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn create_table_index() -> Result<()> {
    let pool = CLIENT.clone();
    let mut tx = pool.begin().await?;
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS file_list_stream_idx on file_list (stream);"#)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS file_list_stream_ts_idx on file_list (stream, min_ts, max_ts);"#,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"CREATE UNIQUE INDEX IF NOT EXISTS file_list_stream_file_idx on file_list (stream, date, file);"#,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(())
}