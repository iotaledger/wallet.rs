foreign_typemap!(
    ($p:r_type) PathBuf => jstring {
        $out = $p.as_path().to_str();
    };
    ($p:f_type, option = "NoNullAnnotations", unique_prefix = "/*chrono*/")
        => "/*chrono*/java.nio.file.Path" "$out = java.nio.file.Paths.get($p);";
);

// Yikes!
foreign_typemap!(
    ($p:r_type) Option<bool> => jshort {
        $out = match $p {
            Some(x) => if x { 1 } else { 0 },
            None => -1,
        };
    };
    ($p:f_type) => "java.util.Optional<java.lang.Boolean>" r#"
        $out;
        if ($p == -1 ) {
            $out = java.util.Optional.empty();
        } else {
            $out = java.util.Optional.of(new java.lang.Boolean($p == 1 ? true : false));
        }
"#;
);

foreign_typemap!(
    ($p:r_type) &u64 => u64 {
        $out = *($p);
    };
);

foreign_typemap!(
    ($p:r_type) &str => PathBuf {
        $out = PathBuf::from($p);
    };
);

foreign_typemap!(
    ($p:r_type) u128 => jobject {
        let data = $p.to_ne_bytes();
        let size = data.len();
        let arr: jbyteArray = (**env)->NewByteArray(env, size);
        (**env)->SetByteArrayRegion(env, arr, 0, size, data);
        
        let clazz: jclass = swig_jni_find_class!(U64_TO_BIGINT, "java.math.BigInteger");
        assert!(!jcls.is_null());
        constructor = (*env)->GetMethodID(env, clazz, "<init>", "([B)V");
        assert!(!constructor.is_null());
        object = (**env)->NewObject(env, clazz, constructor, arr);

        $out = object;
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
        let temp = <u64 as ::std::convert::TryFrom<i64>>::try_from($p)
            .expect("Duration: milleseconds to u64 convert error (number too big?)");
        $out = Duration::from_nanos(temp);
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

// These are dumb...
//TODO: Find out why this typemap doesnt work:
// https://github.com/Dushistov/flapigen-rs/blob/5f248cfdeccd15b70685f810d978567891bc78d3/macroslib/src/java_jni/jni-include.rs#L433
foreign_typemap!(
    ($p:r_type) Vec<Message> <= internal_aliases::JForeignObjectsArray<Message> {
        $out = jobject_array_to_vec_of_objects(env, $p);
    };
    ($p:f_type, option = "NoNullAnnotations") <= "Message[]";
    ($p:f_type, option = "NullAnnotations") <= "@NonNull Message[]";
);

foreign_typemap!(
    ($p:r_type) Vec<Address> <= internal_aliases::JForeignObjectsArray<Address> {
        $out = jobject_array_to_vec_of_objects(env, $p);
    };
    ($p:f_type, option = "NoNullAnnotations") <= "Address[]";
    ($p:f_type, option = "NullAnnotations") <= "@NonNull Address[]";
);
