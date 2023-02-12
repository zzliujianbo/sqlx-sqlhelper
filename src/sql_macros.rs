/// 构造一个元组对象(&str, MySqlArguments)
/// # Examples
///
/// ```
/// let (sql, args) = sql_args!("id = ? AND name = ? AND age = ?", id, &name, age);
/// ```
#[macro_export]
macro_rules! sql_args {

    ($sql:expr) => {
        sql_args!($sql,);
    };

    ($sql:expr, $($args:expr),*) => {{
        let mut mysql_args = sqlx::mysql::MySqlArguments::default();
        $(mysql_args.add($args);)*
        ($sql, mysql_args)
    }};
}
