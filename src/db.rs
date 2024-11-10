use once_cell::sync::Lazy;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result};
use std::error;
use std::fmt;
use std::time::SystemTime;

pub const EXPIRE_SECONDS: u64 = 86400;

#[derive(Debug)]
pub enum Error {
    RusqliteError(rusqlite::Error),
    R2d2Error(r2d2::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::RusqliteError(e) => write!(f, "Rusqlite error: {e}"),
            Error::R2d2Error(e) => write!(f, "R2d2 error: {e}"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::RusqliteError(e) => Some(e),
            Error::R2d2Error(e) => Some(e),
        }
    }
}

// Implement `From` traits for automatic error conversion
impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::RusqliteError(err)
    }
}

impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Error {
        Error::R2d2Error(err)
    }
}

// Type aliases for convenience
pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

// Create a static Lazy instance of the connection pool
static DB_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    let manager = SqliteConnectionManager::file("cpu_stats.db");
    Pool::new(manager).expect("Failed to create pool")
});

pub fn get_connection() -> Result<SqlitePooledConnection, Error> {
    DB_POOL.get().map_err(Error::from)
}

// Function to initialize the database (create tables)
pub fn init() -> Result<(), Error> {
    let conn = get_connection()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stats (
            timestamp   INTEGER PRIMARY KEY,
            cpu_usage   REAL
        )",
        [],
    )?;
    Ok(())
}

// Function to insert CPU usage into the database
pub fn insert_cpu_usage(cpu_usage: f32) -> Result<(), Error> {
    let conn = get_connection()?;
    let timestamp_u64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let timestamp = i64::try_from(timestamp_u64)
        .unwrap_or_else(|_| panic!("Timestamp is too large to fit in an i64"));
    conn.execute(
        "INSERT INTO stats (timestamp, cpu_usage) VALUES (?1, ?2)",
        params![timestamp, cpu_usage],
    )?;
    Ok(())
}

// Function to retrieve all CPU usage stats from the database
pub fn get_all_stats() -> Result<Vec<(i64, f32)>, Error> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("SELECT timestamp, cpu_usage FROM stats ORDER BY timestamp DESC LIMIT 500")?;
    let stats_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,  // timestamp
            row.get::<_, f32>(1)?,  // cpu_usage
        ))
    })?;

    let mut stats = Vec::new();
    for stat in stats_iter {
        stats.push(stat?);
    }
    Ok(stats)
}

pub fn expire_records() -> Result<(), Error> {
    let conn = get_connection()?;
    let q = format!(
        "DELETE FROM stats WHERE timestamp < (unixepoch() - {})",
        EXPIRE_SECONDS
    );
    let mut stmt = conn.prepare(&q)?;
    let _ = match stmt.execute([]) {
        Ok(_) => eprintln!("Expiration Successful"),
        Err(e) => eprintln!("An error occurred: {e}")
    };
    Ok(())
}
