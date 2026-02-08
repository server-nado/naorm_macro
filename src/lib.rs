mod table_create;
use proc_macro::TokenStream;
use proc_macro_error::{abort, emit_error};
use quote::{format_ident, quote, ToTokens};
use syn::parse2;

use syn::MetaNameValue;
use syn::{parse_macro_input, DeriveInput, Expr, Fields, LitStr, Meta};

#[proc_macro_derive(NaormReg, attributes(naorm_cfg))]
pub fn naorm(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    let mut table_name: Option<String> = None;
    let mut db_name: Option<String> = None;
    let mut table_type: Option<String> = None;
    let mut driver: Option<String> = None;

    input.attrs.iter().for_each(|attr| {
        if attr.meta.path().is_ident("naorm_cfg") {
            match &attr.meta {
                Meta::List(data) => {
                    // println!("Found naorm_cfg attribute: {:?}", data);
                    // data.tokins 是 parse2:TokenStream ， 强行转换成 MetaNameValue 来解析 key=value 形式的配置
                    if let Ok(nv) = parse2::<MetaNameValue>(data.tokens.clone()) {
                        let ident_str = match nv.path.segments.first() {
                            Some(seg) => seg.ident.to_string(),
                            None => "".to_string(),
                        };
                        // println!("{:?}{:?}", ident_str, &nv.value.lit);
                        match &nv.value {
                            Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                    let val_str = lit_str.value();
                                    match ident_str.as_str() {
                                        "table_name" => table_name = Some(val_str),
                                        "db_name" => db_name = Some(val_str),
                                        "table_type" => table_type = Some(val_str),
                                        "driver" => driver = Some(val_str),
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        println!(
                            "Failed ========== to parse naorm_cfg tokens: {:?}",
                            data.tokens
                        );
                    }
                }

                _ => (),
            };
        }
    });

    //get fields from struct, only support named fields for now (struct with named fields)
    let struct_ident = &input.ident;
    let table_lit =
        table_create::to_snake_case(&table_name.unwrap_or_else(|| struct_ident.to_string()));
    let db_lit = db_name.unwrap_or_else(|| "".to_string());

    let table_type_lit = table_type.unwrap_or_else(|| "".to_string());

    let mut field_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => fields,
        _ => {
            abort!(input.ident, "Naorm can only be derived for structs");
        }
    };
    let fields = match fields {
        Fields::Named(named) => named,
        _ => {
            abort!(
                input.ident,
                "Naorm can only be derived for structs with named fields"
            );
        }
    };

    // name,name_type, value_type, option, default_value
    let mut fields_type: Vec<(String, syn::Ident, syn::Ident, bool, String)> = Vec::new();
    let mut pk: String = String::new();
    let mut pk_auto_inc: bool = false;
    let mut pk_type: Option<String> = None;
    let mut insert_fields: Vec<String> = Vec::new();
    // (name, ident, field_type_ident, is_option)
    let mut insert_field_meta: Vec<(String, syn::Ident, syn::Ident, bool)> = Vec::new();
    for field in &fields.named {
        let ident = field.ident.as_ref().unwrap();
        let mut ty_name = "_".to_string();
        let mut is_option = false;
        let mut is_auto_increment = false;
        let mut is_primary_key = false;
        let mut default_value: Option<String> = None;
        // println!("Named Field: {:?}, Type: {:?}", ident, field);

        match &field.ty {
            syn::Type::Path(tp) => {
                if let Some(seg) = tp.path.segments.last() {
                    let seg_ident = seg.ident.to_string();
                    if seg_ident == "Option" {
                        // try extract inner type
                        if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                            if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_tp))) =
                                ab.args.first()
                            {
                                if let Some(inner_seg) = inner_tp.path.segments.last() {
                                    ty_name = inner_seg.ident.to_string();
                                }
                            }
                        }
                        is_option = true;
                    } else {
                        ty_name = seg_ident;
                    }
                }
            }
            _ => {
                // fallback: stringify the type
                ty_name = quote! { #field.ty }.to_string();
            }
        }

        // println!("Named Field: {:?}", field);
        field.attrs.iter().for_each(|attr| {
            if attr.meta.path().is_ident("naorm_cfg") {
                let toks = attr.to_token_stream().to_string();

                // let toks = attr.tokens.to_string();
                if toks.contains("auto_increment") {
                    is_auto_increment = true;
                    pk = ident.to_string();
                    pk_auto_inc = true;
                    pk_type = Some(ty_name.clone());
                }
                if toks.contains("primary_key") {
                    is_primary_key = true;
                    if pk.is_empty() {
                        pk = ident.to_string();
                        pk_type = Some(ty_name.clone());
                    }
                }
                if toks.contains("default") {
                    if let Some(def_idx) = toks.find("default") {
                        if let Some(first_q_rel) = toks[def_idx..].find('"') {
                            let first_q = def_idx + first_q_rel;
                            if let Some(second_q_rel) = toks[first_q + 1..].find('"') {
                                let second_q = first_q + 1 + second_q_rel;
                                let val = &toks[first_q + 1..second_q];
                                default_value = Some(val.to_string());
                            }
                        }
                    }
                }
            }
        });
        if default_value.is_none() {
            match ty_name.as_str() {
                "String" => default_value = Some("".to_string()),
                "u64" | "u32" | "u8" | "i64" | "i32" | "usize" | "isize" | "f64" | "f32" => {
                    default_value = Some("0".to_string())
                }
                "float" | "float32" | "float64" => default_value = Some("0.0".to_string()),
                "bool" => default_value = Some("false".to_string()),
                _ => {}
            }
        }

        let field_name = ident.to_string();
        let field_ident_token =
            syn::Ident::new(&field_name.clone(), proc_macro2::Span::call_site());
        let name_lit = LitStr::new(&ident.to_string(), proc_macro2::Span::call_site());
        let ty_lit = LitStr::new(&ty_name, proc_macro2::Span::call_site());
        let is_option_lit = if is_option {
            quote! { true }
        } else {
            quote! { false }
        };
        let is_auto_inc_lit = if is_auto_increment {
            quote! { true }
        } else {
            quote! { false }
        };
        let is_pk_lit = if is_primary_key {
            quote! { true }
        } else {
            quote! { false }
        };
        let default_str = default_value.unwrap_or_else(|| "".to_string());
        let default_lit = LitStr::new(&default_str, proc_macro2::Span::call_site());
        field_tokens.push( quote! { (#name_lit, #ty_lit, #is_option_lit, #is_auto_inc_lit, #is_pk_lit, #default_lit) },);

        if !is_auto_increment {
            insert_fields.push(field_name.clone());
            insert_field_meta.push((
                field_name.clone(),
                field_ident_token.clone(),
                format_ident!("as_{}", table_create::to_snake_case(ty_name.as_str())),
                is_option,
            ));
        }
        fields_type.push((
            field_name.clone(),
            field_ident_token.clone(),
            format_ident!("as_{}", table_create::to_snake_case(ty_name.as_str())),
            is_option,
            default_str,
        ));
    }

    let insert_sql_string = if insert_fields.is_empty() {
        format!("INSERT INTO {} DEFAULT VALUES", table_lit)
    } else {
        let placeholders = vec!["?"; insert_fields.len()].join(", ");
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_lit,
            insert_fields.join(", "),
            placeholders
        )
    };
    let insert_sql_lit = LitStr::new(&insert_sql_string, proc_macro2::Span::call_site());

    let bind_tokens: Vec<proc_macro2::TokenStream> = insert_field_meta
        .iter()
        .map(|(_name, ident, field_type, is_option)| {
            if *is_option {
                if field_type.to_string() == "as_string" {
                    quote! { .bind(self.#ident.as_deref()) }
                } else {
                    quote! { .bind(self.#ident.as_ref()) }
                }
            } else {
                if field_type.to_string() == "as_string" {
                    quote! { .bind(self.#ident.as_str()) }
                } else {
                    quote! { .bind(&self.#ident) }
                }
            }
        })
        .collect();

    let update_bind_tokens: Vec<proc_macro2::TokenStream> = {
        let mut toks: Vec<proc_macro2::TokenStream> = Vec::new();
        for (_name, ident, field_type, is_option) in &insert_field_meta {
            if *is_option {
                if field_type.to_string() == "as_string" {
                    toks.push(quote! { .bind(self.#ident.as_deref()) });
                } else {
                    toks.push(quote! { .bind(self.#ident.as_ref()) });
                }
            } else {
                if field_type.to_string() == "as_string" {
                    toks.push(quote! { .bind(self.#ident.as_str()) });
                } else {
                    toks.push(quote! { .bind(&self.#ident) });
                }
            }
        }
        if !pk.is_empty() {
            // let pk_field_type_opt = fields_type
            //     .iter()
            //     .find(|(n, _id, _ft, _opt, _d)| n == &pk)
            //     .map(|(_n, _id, ft, _opt, _d)| ft.to_string());
            let pk_ident = syn::Ident::new(&pk, proc_macro2::Span::call_site());
            // bind primary key (handle Option and String similarly if necessary)
            // try to find pk meta in insert_field_meta to know is_option/field_type
            let pk_meta = insert_field_meta
                .iter()
                .find(|(n, _id, _ft, _opt)| n == &pk);
            match pk_meta {
                Some((_n, _id, ft, is_opt)) => {
                    if *is_opt {
                        if ft.to_string() == "as_string" {
                            toks.push(quote! { .bind(self.#pk_ident.as_deref()) });
                        } else {
                            toks.push(quote! { .bind(self.#pk_ident.as_ref()) });
                        }
                    } else {
                        if ft.to_string() == "as_string" {
                            toks.push(quote! { .bind(self.#pk_ident.as_str()) });
                        } else {
                            toks.push(quote! { .bind(&self.#pk_ident) });
                        }
                    }
                }
                None => toks.push(quote! { .bind(&self.#pk_ident) }),
            }
        }
        toks
    };

    let update_sql_string = if insert_fields.is_empty() {
        format!("-- No fields to update for table {}", table_lit)
    } else {
        let set_clauses: Vec<String> = insert_fields
            .iter()
            .map(|field| format!("{} = ?", field))
            .collect();
        format!(
            "UPDATE {} SET {} WHERE {} = ?",
            table_lit,
            set_clauses.join(", "),
            pk
        )
    };

    let update_sql_lit = LitStr::new(&update_sql_string, proc_macro2::Span::call_site());

    let delete_sql_string = if pk.is_empty() {
        format!("-- No PK for table {}", table_lit)
    } else {
        format!("DELETE FROM {} WHERE {} = ?", table_lit, pk)
    };
    let delete_sql_lit = LitStr::new(&delete_sql_string, proc_macro2::Span::call_site());

    let delete_bind_tokens: Vec<proc_macro2::TokenStream> = if pk.is_empty() {
        Vec::new()
    } else {
        let pk_ident = syn::Ident::new(&pk, proc_macro2::Span::call_site());
        match insert_field_meta
            .iter()
            .find(|(n, _id, _ft, _opt)| n == &pk)
        {
            Some((_n, _id, ft, is_opt)) => {
                if *is_opt {
                    if ft.to_string() == "as_string" {
                        vec![quote! { .bind(self.#pk_ident.as_deref()) }]
                    } else {
                        vec![quote! { .bind(self.#pk_ident.as_ref()) }]
                    }
                } else {
                    if ft.to_string() == "as_string" {
                        vec![quote! { .bind(self.#pk_ident.as_str()) }]
                    } else {
                        vec![quote! { .bind(&self.#pk_ident) }]
                    }
                }
            }
            None => vec![quote! { .bind(&self.#pk_ident) }],
        }
    };

    // select tokens: build SELECT_SQL and optional bind tokens (no binds for simple select *)
    let select_fields: Vec<String> = fields_type
        .iter()
        .map(|(n, _id, _ft, _opt, _d)| n.clone())
        .collect();
    let select_sql_string = if select_fields.is_empty() {
        format!("SELECT * FROM {}", table_lit)
    } else {
        format!("SELECT {} FROM {}", select_fields.join(", "), table_lit)
    };
    let select_sql_lit = LitStr::new(&select_sql_string, proc_macro2::Span::call_site());

    // For now there are no default select bind tokens (specific query methods may add them later)

    // prepare pk-related tokens for generated save method

    let pk_auto_inc_lit = if pk_auto_inc {
        quote! { true }
    } else {
        quote! { false }
    };

    // driver
    let mut driver_lit = quote! { sqlx::Sqlite};
    let mut driver_arg_lit = quote! { sqlx::sqlite::SqliteArguments};
    let mut driver_row_lit = quote! { sqlx::sqlite::SqliteRow};
    match driver {
        Some(ref d) if d.to_lowercase() == "sqlite" => {
            // already set
            driver_lit = quote! { sqlx::Sqlite};
            driver_arg_lit = quote! { sqlx::sqlite::SqliteArguments};
            driver_row_lit = quote! { sqlx::sqlite::SqliteRow};
        }
        Some(ref d) if d.to_lowercase() == "mysql" => {
            driver_lit = quote! { sqlx::MySql};
            driver_arg_lit = quote! { sqlx::mysql::MySqlArguments};
            driver_row_lit = quote! { sqlx::mysql::MySqlRow};
        }
        Some(ref d) if d.to_lowercase() == "postgres" => {
            driver_lit = quote! { sqlx::Postgres};
            driver_arg_lit = quote! { sqlx::postgres::PgArguments};
            driver_row_lit = quote! { sqlx::postgres::PgRow};
        }
        Some(ref d) => {
            emit_error!(input.ident, format!("Unsupported driver specified: {}", d));
        }
        None => {
            // default to sqlite
        }
    }

    // 生成代码：保留原始 struct，并为其生成常量/方法
    let expanded = quote! {
        impl #struct_ident {
            pub const PK : &'static str = #pk;
            pub const PK_AUTO_INCREMENT : bool = #pk_auto_inc_lit;
            pub const NAORM_TABLE: &'static str = #table_lit;
            pub const NAORM_DB: &'static str = #db_lit;
            pub const NAORM_TABLE_TYPE: &'static str = #table_type_lit;
            pub const SELECT_SQL: &'static str = #select_sql_lit;
            pub const INSERT_SQL: &'static str = #insert_sql_lit;
            pub const UPDATE_SQL: &'static str = #update_sql_lit;
            pub const DELETE_SQL: &'static str = #delete_sql_lit;
            // file_name , field_type, is_option, is_auto_increment, is_primary_key, default_value
            pub const NAORM_FIELDS: &'static [(&'static str, &'static str, bool, bool, bool, &'static str)] = &[
                #(#field_tokens),*
            ];
            pub fn insert_query<'q>(&'q mut self) -> sqlx::query::Query<'q, #driver_lit, #driver_arg_lit<'q>> {
                sqlx::query(Self::INSERT_SQL)
                    #(#bind_tokens)*
            }
            pub fn update_query<'q>(&'q mut self) -> sqlx::query::Query<'q, #driver_lit, #driver_arg_lit<'q>> {
                sqlx::query(Self::UPDATE_SQL)
                    #(#update_bind_tokens)*
            }
            pub fn delete_query<'q>(&'q self) -> sqlx::query::Query<'q, #driver_lit, #driver_arg_lit<'q>> {
                sqlx::query(Self::DELETE_SQL)
                    #(#delete_bind_tokens)*
            }

            pub fn all_query() -> sqlx::query::QueryAs<'static, #driver_lit, Self, #driver_arg_lit<'static>>
            where
                Self: for<'r> sqlx::FromRow<'r, #driver_row_lit>,
            {
                sqlx::query_as::<#driver_lit, Self>(Self::SELECT_SQL)
            }
            pub fn filter_query<'q>(
                w: &'q str,
            ) -> sqlx::query::QueryAs<'q, #driver_lit, Self, #driver_arg_lit<'q>>
            where
                Self: for<'r> sqlx::FromRow<'r, #driver_row_lit>,
            {
                sqlx::query_as::<#driver_lit, Self>(w)
            }
        }

    };

    TokenStream::from(expanded)
}
