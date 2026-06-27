use mlua::{Error, ErrorContext, ExternalError, ExternalResult as _, Lua, Result};

#[test]
fn syntax_error() {
    let lua = Lua::new();
    let err = lua.load("this is not valid lua !!!").exec().unwrap_err();
    assert!(matches!(err, Error::SyntaxError { .. }));
}

#[test]
fn runtime_error() {
    let lua = Lua::new();
    let err = lua.load("error('boom')").exec().unwrap_err();
    assert!(matches!(err, Error::RuntimeError(_)));
    assert!(err.to_string().contains("boom"));
}

#[test]
fn error_from_rust_function() {
    let lua = Lua::new();
    let f = lua
        .create_function(|_, ()| -> Result<()> {
            Err(Error::RuntimeError("something went wrong".into()))
        })
        .unwrap();
    lua.globals().set("fail", f).unwrap();
    let err = lua.load("fail()").exec().unwrap_err();
    assert!(err.to_string().contains("something went wrong"));
}

#[test]
fn external_error_conversion() {
    #[derive(Debug)]
    struct MyError(String);
    impl std::fmt::Display for MyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MyError: {}", self.0)
        }
    }
    impl std::error::Error for MyError {}

    let lua = Lua::new();
    let f = lua
        .create_function(|_, ()| -> Result<()> {
            Err(MyError("custom".into()).into_lua_err())
        })
        .unwrap();
    lua.globals().set("custom_fail", f).unwrap();
    let err = lua.load("custom_fail()").exec().unwrap_err();
    assert!(err.to_string().contains("MyError"));
}

#[test]
fn external_result_ok() {
    let r: std::result::Result<i32, std::num::ParseIntError> = "42".parse();
    let lua_r: Result<i32> = r.into_lua_err();
    assert_eq!(lua_r.unwrap(), 42);
}

#[test]
fn external_result_err() {
    let r: std::result::Result<i32, std::num::ParseIntError> = "not_a_number".parse();
    let lua_r: Result<i32> = r.into_lua_err();
    assert!(lua_r.is_err());
}

#[test]
fn error_context_wraps_message() {
    let base: Result<i64> = Err(Error::RuntimeError("base error".into()));
    let err = base.context("extra context").unwrap_err();
    assert!(err.to_string().contains("extra context"));
}


#[test]
fn pcall_catches_error() {
    let lua = Lua::new();
    let result: Result<()> = lua.load("error('caught')").exec();
    assert!(result.is_err());
}