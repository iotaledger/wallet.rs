foreign_typemap!(
    ($p:r_type) PathBuf => jstring {
        $out = $p.as_path().to_str();
    };
    ($p:f_type, option = "NoNullAnnotations", unique_prefix = "/*chrono*/")
        => "/*chrono*/java.nio.file.Path" "$out = java.nio.file.Paths.get($p);";
);

foreign_typemap!(
    ($p:r_type) &str => PathBuf {
        $out = PathBuf::from($p);
    };
);

foreign_typemap!(
    ($p:r_type) <T> Result<T> => swig_i_type!(T) {
        $out = match $p {
            Ok(x) => {
                swig_from_rust_to_i_type!(T, x, ret)
                ret
            }
            Err(err) => {
                let msg = err.to_string();
                let exception_class = match err {
                    _ => swig_jni_find_class!(WALLET_BASE_EXCEPTION, "java/lang/Error"),
                };
                jni_throw(env, exception_class, &msg);
                return <swig_i_type!(T)>::jni_invalid_value();
            }
        };
    };
    ($p:f_type, unique_prefix="/*wallet::error::Result<swig_subst_type!(T)>*/") => "/*wallet::error::Result<swig_subst_type!(T)>*/swig_f_type!(T)"
        "swig_foreign_from_i_type!(T, $p)";
);

// Duration
foreign_typemap!(
    ($p:r_type) Duration => jlong {
        $out = $p.as_nanos();
    };
);

//TODO: Make sure duration doenst cross the i64 limit
foreign_typemap!(
    ($p:r_type) jlong => Duration {
        let temp = <u64 as ::std::convert::TryFrom<i64>>::try_from($p);
        $out = Duration::from_nanos(temp.unwrap());
    };
);

fn jstring_array_to_vec_of_string(
    env: *mut JNIEnv,
    arr: internal_aliases::JStringObjectsArray,
) -> Vec<String> {
    let length = unsafe { (**env).GetArrayLength.unwrap()(env, arr) };
    
    let len = <usize as ::std::convert::TryFrom<jsize>>::try_from(length)
        .expect("invalid jsize, in jsize => usize conversation");
    let mut result = Vec::with_capacity(len);
    for i in 0..length {
        let native: String = unsafe {
            let obj: jstring = (**env).GetObjectArrayElement.unwrap()(env, arr, i);
            if (**env).ExceptionCheck.unwrap()(env) != 0 {
                panic!("Failed to retrieve element {} from this `jobjectArray'", i);
            }
            let jstr = JavaString::new(env, obj);
            jstr.to_str().to_string()
        };
        result.push(native);
    }

    result
}

foreign_typemap!(
    ($p:r_type) Vec<String> <= internal_aliases::JStringObjectsArray {
        $out = jstring_array_to_vec_of_string(env, $p);
    };
    ($p:f_type, option = "NoNullAnnotations") <= "java.lang.String []";
    ($p:f_type, option = "NullAnnotations")
                  <= "@NonNull java.lang.String []";
);
