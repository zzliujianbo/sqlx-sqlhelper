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
