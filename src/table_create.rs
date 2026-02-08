pub fn to_snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 && !out.ends_with('_') {
                out.push('_');
            }
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

// pub fn CreateSqlite(
//     table_name: &str,
//     fields: Vec<(String, syn::Ident, bool, bool, bool, String)>,
// ) -> String {
//     let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (", table_name);
//     let mut field_defs: Vec<String> = Vec::new();
//     for (name, ty, is_option, is_auto_inc, is_pk, default) in fields.iter() {
//         let type_name = match ty.to_string().as_str() {
//             "String" => "TEXT",
//             "u64" | "u32" | "i64" | "i32" | "usize" | "isize" => "INTEGER",
//             "f64" | "f32" => "REAL",
//             "bool" => "BOOLEAN",
//             _ => "BLOB", // Default to BLOB for unsupported types
//         };
//         let mut field_def = format!("{} {}", to_snake_case(name.as_str()), type_name);

//         if *is_pk {
//             field_def.push_str(" PRIMARY KEY");
//         }
//         if *is_auto_inc {
//             field_def.push_str(" AUTOINCREMENT");
//         }
//         if !*is_option {
//             field_def.push_str(" NOT NULL");
//         }
//         if !default.is_empty() {
//             field_def.push_str(&format!(" DEFAULT {}", default));
//         }
//         field_defs.push(field_def);
//     }
//     sql.push_str(field_defs.join(",\n ").as_str());
//     sql.push(')');
//     sql
// }
