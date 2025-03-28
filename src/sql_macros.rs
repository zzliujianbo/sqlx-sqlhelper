/// 构造一个元组对象(&str, MySqlArguments)
///
/// 需要引入`sqlx::Arguments`
///
/// ```
/// use sqlx::Arguments;
/// ```
///
/// # Examples
///
/// ```
/// use sqlx::Arguments;
/// let (sql, args) = sql_args!("id = ? AND name = ? AND age = ?", id, &name, age);
/// ```
#[macro_export]
macro_rules! sql_args {

    ($sql:expr) => {
        sql_args!($sql,);
    };

    ($sql:expr, $($args:expr),*) => {{
        let mut mysql_args = sqlx::mysql::MySqlArguments::default();
        $(match mysql_args.add($args){
            Ok(_) => {},
            Err(e) => {
                warn!("add mysql args error: {}, {}", e, $args);
            }
        };)*
        ($sql, mysql_args)
    }};
}

#[macro_export]
macro_rules! query_one {
    ($sql:expr) => {
        query_one!($sql,);
    };
    ($sql:expr, $($args:expr),*) => {{
        let (sql, args) = sql_args!($sql, $($args),*);
        sqlx::query_as_with::<_, Self, sqlx::mysql::MySqlArguments>(&sql, args)
            .fetch_one(&*db::POOL)
            .await
    }};
}

#[macro_export]
macro_rules! query_all {
    ($sql:expr) => {
        query_all!($sql,)
    };
    ($sql:expr, $($args:expr),*) => {{
        let (sql, args) = sql_args!($sql, $($args),*);
        sqlx::query_as_with::<_, Self, sqlx::mysql::MySqlArguments>(&sql, args)
            .fetch_all(&*db::POOL)
            .await
    }};
}

#[macro_export]
macro_rules! execute {
    ($sql:expr) => {
        execute!($sql,)
    };
    ($sql:expr, $($args:expr),*) => {{
        let (sql, args) = sql_args!($sql, $($args),*);
        sqlx::query_with::<_, sqlx::mysql::MySqlArguments>(&sql, args)
            .execute(&*db::POOL)
            .await
    }};
}

#[macro_export]
macro_rules! tran_execute {
    ($tran:expr, $sql:expr) => {
        execute!($tran, $sql,)
    };
    ($tran:expr, $sql:expr, $($args:expr),*) => {{
        let (sql, args) = sql_args!($sql, $($args),*);
        sqlx::query_with::<_, sqlx::mysql::MySqlArguments>(&sql, args)
            .execute(&mut **$tran)
            .await
    }};
}

