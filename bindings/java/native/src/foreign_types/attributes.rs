pub fn class_partial_eq(code: &mut Vec<u8>, class_name: &str){
    let needle = format!("class {} {{", class_name);
    let class_pos = code
        .windows(needle.len())
        .position(|window| window == needle.as_bytes())
        .expect("Can not find begin of class");
    let insert_pos = class_pos + needle.len();
    code.splice(
        insert_pos..insert_pos,
        format!(
                r#"
    public boolean equals(Object obj) {{
        boolean equal = false;
        if (obj instanceof {class})
        equal = (({class})obj).rustEq(this);
        return equal;
    }}
    public int hashCode() {{
        return (int)mNativeObj;
    }}
"#,
            class = class_name
        )
        .as_bytes()
        .iter()
        .copied(),
    );
}

pub fn class_getters(code: &mut Vec<u8>, class_name: &str){
    
}

pub fn class_setters(code: &mut Vec<u8>, class_name: &str){
    
}