use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Attribute, Field, Fields, ItemStruct, Visibility};

use crate::{DEFAULT_CREATE_TIME_NAME, DEFAULT_ID_NAME, DEFAULT_UPDATE_TIME_NAME};

pub fn impl_sql_helper(ast: &ItemStruct) -> TokenStream {
    //初始化model，默认model实现，分页model实现等。初始化获取一个model

    let mut filed_vec = Vec::new();
    ast.fields.iter().for_each(|field| {
        if is_vis_public_crate(&field.vis)
            && is_base_type(field)
            && !field_attr_exists(field, DEFAULT_ID_NAME)
        {
            if let Some(ident) = &field.ident {
                if ident == DEFAULT_ID_NAME {
                    return;
                }
            }
            filed_vec.push(field);
        }
    });

    let table_filed_name_vec = filed_vec
        .iter()
        //.map(|field| field.ident.as_ref().unwrap().to_string())
        .map(|field| get_table_field_name(field))
        //.map(get_table_field_name)
        .collect::<Vec<_>>();

    let struct_name = &ast.ident;
    let self_ident = format_ident!("self");
    //let varname = format_ident!("_{}", ident);
    let struct_var_name = format_ident!("{}", struct_name.to_string().to_snake_case());
    let id = get_ident(&ast.fields, DEFAULT_ID_NAME);
    let create_time = get_ident(&ast.fields, DEFAULT_CREATE_TIME_NAME);
    let update_time = get_ident(&ast.fields, DEFAULT_UPDATE_TIME_NAME);
    let pool = quote!(&*db::POOL);
    let query = quote!(sqlx::query);
    let query_as = quote!(sqlx::query_as::<_, Self>);

    let select_base_sql = format!(
        "SELECT {}, {} FROM {}",
        id,
        table_filed_name_vec.join(", ").trim_end(),
        struct_var_name
    );
    let count_base_sql = format!("SELECT count(1) FROM {}", struct_var_name);

    //查找函数
    let find_sql = format!("{} WHERE {} = ?", select_base_sql, id);
    let find_fn = quote!(
        pub async fn find(#id: i32) -> Result<Self, sqlx::Error> {
            //sqlx::query_as::<_, Self>(&format!(
            //    "SELECT * FROM {} WHERE id = ?",
            //    stringify!(#struct_name)
            //))
            #query_as(#find_sql)
            .bind(#id)
            .fetch_one(#pool)
            .await
        }
    );

    //列表函数
    let list_fn = quote!(
        pub async fn list() -> Result<Vec<Self>, sqlx::Error> {
            #query_as(#select_base_sql)
            .fetch_all(#pool)
            .await
        }
    );

    //删除函数
    let delete_sql = format!("DELETE FROM {} WHERE {} = ?", struct_var_name, id);
    let delete_fn = quote!(
        pub async fn delete(&self) -> Result<bool, sqlx::Error> {
            #query(#delete_sql)
            .bind(self.#id)
            .execute(#pool)
            .await
            .map(|f| f.rows_affected() > 0)
        }
    );

    //新增函数
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES({})",
        struct_var_name,
        table_filed_name_vec.join(", ").trim_end(),
        "?, "
            .repeat(filed_vec.len())
            .trim_end()
            .trim_end_matches(',')
    );

    let insert_bind_quote_vec = fileds_to_bind_quote(&self_ident, &filed_vec);
    let insert_auto_time_quote = get_auto_time_quote(&self_ident, Some(&create_time), &update_time);
    //pub async fn add(#struct_var_name:&Self) -> Result<Self, sqlx::Error> {
    //    let sql = #add_sql;
    //    let last_id = #query(sql)
    //    #(#add_bind_quote_vec)*
    //    .execute(#pool).await?.last_insert_id();
    //    Self::find(last_id as i32).await
    //}
    let insert_fn = quote!(
        pub async fn insert(&self) -> Result<Self, sqlx::Error> {
            let sql = #insert_sql;
            let last_id = #query(sql)
            #(#insert_bind_quote_vec)*
            .execute(#pool).await?.last_insert_id();
            Self::find(last_id as i32).await
        }

        /// 如果定义的`create_time`，`update_time`字段是`Default::default()`默认值，则更新为当前时间
        ///
        /// `Default::default()`一般为`1970-01-01T00:00:00`等
        pub async fn insert_auto_time(&mut self) -> Result<Self, sqlx::Error> {
            #insert_auto_time_quote
            self.insert().await
        }

    );

    //更新函数
    let update_sql = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        struct_var_name,
        table_filed_name_vec
            .iter()
            .map(|filed_str| format!("{} = ?", filed_str))
            .collect::<Vec<_>>()
            .join(", "),
        id
    );

    let update_bind_quote_vec = fileds_to_bind_quote(&self_ident, &filed_vec);

    let update_auto_time_quote = get_auto_time_quote(&self_ident, None, &update_time);
    let update_fn = quote!(
        pub async fn update(&self) -> Result<bool, sqlx::Error> {
            let sql = #update_sql;
            #query(sql)
            #(#update_bind_quote_vec)*
            .bind(self.#id)
            .execute(#pool).await.map(|f|f.rows_affected() > 0)
        }

        /// 如果定义的update_time字段是`Default::default()`默认值，则更新为当前时间
        ///
        /// `Default::default()`一般为`1970-01-01T00:00:00`等
        pub async fn update_auto_time(&mut self) -> Result<bool, sqlx::Error> {
            #update_auto_time_quote
            self.update().await
        }
    );

    //保存或者修改函数
    let save_or_update_fn = quote!(
        /// 调用`save_or_update`方法时有一定风险
        ///
        /// `save_or_update`只是简单判断id是否大于0，大于0则更新，小于等于0则插入。
        ///
        /// 此时如果手动将`id`赋值为大于0时，会出现更新其他数据的情况，请注意这一块。
        pub async fn save_or_update(&self) -> Result<bool, sqlx::Error> {
            match self.#id > 0 {
                true => self.update().await,
                //false => Self::add(self).await.map(|_| true),
                false => self.insert().await.map(|_| true),
            }
        }

        /// 如果定义的update_time字段是`Default::default()`默认值，则更新为当前时间
        ///
        /// Default::default()一般为`1970-01-01T00:00:00`等
        ///
        /// 调用`save_or_update`方法时有一定风险
        ///
        /// `save_or_update`只是简单判断id是否大于0，大于0则更新，小于等于0则插入。
        ///
        /// 此时如果手动将`id`赋值为大于0时，会出现更新其他数据的情况，请注意这一块。
        pub async fn save_or_update_auto_time(&mut self) -> Result<bool, sqlx::Error> {
            match self.#id > 0 {
                true => self.update_auto_time().await,
                //false => Self::add(self).await.map(|_| true),
                false => self.insert_auto_time().await.map(|_| true),
            }
        }
    );

    let mut new_auto_filed_vec = vec![];
    for field in &filed_vec {
        if !field_attr_exists(field, DEFAULT_CREATE_TIME_NAME)
            && !field_attr_exists(field, DEFAULT_UPDATE_TIME_NAME)
        {
            if let Some(ident) = &field.ident {
                if ident == DEFAULT_CREATE_TIME_NAME || ident == DEFAULT_UPDATE_TIME_NAME {
                    continue;
                }
            }

            new_auto_filed_vec.push(field);
        }
    }
    let new_filed_vec = new_auto_filed_vec
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            quote!(#ident: #ty)
        })
        .collect::<Vec<_>>();

    let new_self_filed_vec = new_auto_filed_vec
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            quote!(#ident)
        })
        .collect::<Vec<_>>();

    let new_fn = quote!(
        pub fn new(#(#new_filed_vec),*,#create_time: chrono::NaiveDateTime, #update_time: chrono::NaiveDateTime) -> Self {
            Self{
                #id: 0,
                #(#new_self_filed_vec),*,
                #create_time,
                #update_time
            }
        }
        pub fn new_common(#(#new_filed_vec),*) -> Self {
            Self::new(
                #(#new_self_filed_vec),*,
                chrono::Local::now().naive_local(),
                chrono::Local::now().naive_local()
            )
        }
    );

    let base_page_select_sql = format!("{} WHERE {{}} LIMIT {{}}, {{}}", select_base_sql);

    let base_page_fn = quote!(
        pub async fn base_page(
            page_index: i32,
            page_size: i32,
            where_sql: &str,
            args: sqlx::mysql::MySqlArguments,
        ) -> Result<(Vec<Self>, i32, i32, i32), sqlx::Error> {
            let mut index = page_index - 1;
            if index < 0 {
                index = 0;
            }
            let rows = page_size;


            let (count,) = Self::base_count(where_sql, args.clone()).await?;

            let arr = match count > 0 {
                true => {
                    let sql = format!(
                        #base_page_select_sql,
                        where_sql,
                        index * rows,
                        rows
                    );
                    sqlx::query_as_with::<_, Self, sqlx::mysql::MySqlArguments>(&sql, args)
                        .fetch_all(#pool)
                        .await?
                }
                false => Vec::new(),
            };

            //总共有几页
            let total_page = (count as f32 / page_size as f32).ceil();

            Ok((arr, count, index + 1, total_page as i32))
        }
    );

    //总数函数
    let base_count_sql = format!("{} WHERE {{}}", count_base_sql);
    let base_count_fn = quote!(
        pub async fn base_count(
            where_sql: &str,
            args: sqlx::mysql::MySqlArguments,
        ) -> Result<(i32,), sqlx::Error> {
            let count_sql = format!(#base_count_sql, where_sql);
            sqlx::query_as_with::<_, (i32,), sqlx::mysql::MySqlArguments>(
                &count_sql,
                args,
            )
            .fetch_one(#pool)
            .await
        }
    );

    let gen = quote!(
        impl #struct_name {
            #find_fn

            #list_fn

            #delete_fn

            #insert_fn

            #update_fn

            #save_or_update_fn

            #new_fn

            #base_page_fn

            #base_count_fn
        }
    );
    gen.into()
}

fn fileds_to_bind_quote(struct_ident: &Ident, fields: &Vec<&Field>) -> Vec<TokenStream2> {
    fields
        .iter()
        .map(|field| filed_to_bind_quote(&struct_ident, field))
        .collect()
}

fn filed_to_bind_quote(struct_ident: &Ident, filed: &Field) -> TokenStream2 {
    //此处&filed.ty会自动解引用，不用在通过ref关键字借用typePath。
    //参考：https://rust-lang.github.io/rfcs/2005-match-ergonomics.html
    //https://doc.rust-lang.org/std/keyword.ref.html
    //Type::Path(ref p) = field.ty
    let syn::Type::Path(type_path) = &filed.ty else {
        return quote!();
    };
    let Some(filed_var_name) = &filed.ident else {
        return quote!();
    };
    //let filed_quote = type_path.path.is_ident("NaiveDateTime");
    //eprintln!("{:?}", extract_type_from_option(&filed.ty));

    let filed_qoute = if extract_type_from_option(&filed.ty).is_some() {
        quote!(#struct_ident.#filed_var_name.as_ref())
    } else {
        //此处判断还有一些问题，如果struct的字段类型不带路径，则能正常判断。
        //比如：pub create_name:NaiveDateTime这种方式声明则没有问题。
        //但是如pub create_name:chrono::NaiveDateTime这种全路径声明字段类型时，type_path.path.get_ident()是获取不到NaiveDateTime类型的
        //最后会导致宏生成的代码出现空的情况
        match type_path.path.get_ident() {
            Some(ident) => {
                if ident == "String" || ident == "Decimal" {
                    quote!(&#struct_ident.#filed_var_name)
                } else {
                    quote!(#struct_ident.#filed_var_name)
                }
            }
            _ => quote!(),
        }
    };
    quote!(
        .bind(#filed_qoute)
    )
}

/// 判断是否public字段
fn is_vis_public_crate(vis: &Visibility) -> bool {
    match vis {
        Visibility::Public(_) | Visibility::Crate(_) => true,
        _ => false,
    }
}
/// 判断是否为基础类型，非struct类型
fn is_base_type(field: &Field) -> bool {
    match (&field.ty, &field.ident) {
        (syn::Type::Path(_), Some(_)) => true,
        _ => false,
    }
}

/// 判断字段属性是否存在
fn field_attr_exists(field: &Field, attr_name: &str) -> bool {
    get_field_attr(field, attr_name).is_some()
}

/// 根据属性名字获取字段的属性对象
fn get_field_attr<'a>(field: &'a Field, attr_name: &str) -> Option<(&'a Field, &'a Attribute)> {
    for attribute in field.attrs.iter() {
        if attribute.path.is_ident(attr_name) {
            return Some((field, attribute));
        }
    }
    None
}

/// 根据字段名字获取字段列表中的字段对象
fn get_ident(fields: &Fields, ident_name: &str) -> Ident {
    if let Some(field) = fields
        .iter()
        .find(|field| field_attr_exists(field, ident_name))
    {
        if let Some(ident) = &field.ident {
            return ident.clone();
        }
    }
    format_ident!("{}", ident_name)
}

/// 获取表字段名字
fn get_table_field_name(field: &Field) -> String {
    if let Some((_, attr)) = get_field_attr(field, "field_name") {
        // for attribute in field.attrs.iter() {
        //     eprintln!("field attribute: {}", attribute.tokens);
        // }

        //参考：https://users.rust-lang.org/t/how-to-parse-the-value-of-a-macros-helper-attribute/39882
        //https://docs.rs/syn/latest/syn/struct.Attribute.html
        let lit = attr.parse_args::<syn::LitStr>();
        if let Ok(s) = lit {
            return s.value();
        }
    }
    field.ident.as_ref().unwrap().to_string()
}

fn get_auto_time_quote(
    struct_ident: &Ident,
    create_time: Option<&Ident>,
    update_time: &Ident,
) -> TokenStream2 {
    let create_time_quote = if let Some(ct) = create_time {
        get_update_time_quote(struct_ident, ct)
    } else {
        quote!()
    };
    let update_time_quote = get_update_time_quote(struct_ident, update_time);
    quote!(
        #create_time_quote
        #update_time_quote
    )
}

fn get_update_time_quote(struct_ident: &Ident, time_ident: &Ident) -> TokenStream2 {
    quote!(
        //if #struct_ident.#time_ident == Default::default() {
            #struct_ident.#time_ident = chrono::Local::now().naive_local();
        //}
    )
}

/// 获取type是否为Option类型
/// 代码来自于：https://stackoverflow.com/a/56264023
fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, Path, PathArguments, PathSegment};

    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    // TODO store (with lazy static) the vec of string
    // TODO maybe optimization, reverse the order of segments
    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}
