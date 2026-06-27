use mlua::{Lua, Result, Table};

pub fn preload(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set("read", lua.create_async_function(|_, path: String| async move {
        tokio::fs::read_to_string(path).await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?)?;

    t.set("write", lua.create_async_function(|_, (path, content): (String, String)| async move {
        tokio::fs::write(path, content.into_bytes()).await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?)?;

    t.set("exists", lua.create_async_function(|_, path: String| async move {
        tokio::fs::try_exists(path).await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?)?;

    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{LuaOptions, StdLib};
    use tokio::task::LocalSet;

    fn lua() -> Lua {
        Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap()
    }

    fn rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    #[test]
    fn preload_returns_table_with_read_write_exists() {
        let lua = lua();
        let tbl = preload(&lua).unwrap();
        assert!(tbl.get::<mlua::Function>("read").is_ok());
        assert!(tbl.get::<mlua::Function>("write").is_ok());
        assert!(tbl.get::<mlua::Function>("exists").is_ok());
    }

    #[test]
    fn write_and_read_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt").to_str().unwrap().to_string();
        let lua = lua();
        let tbl = preload(&lua).unwrap();
        lua.globals().set("fs", tbl).unwrap();
        lua.globals().set("p", path).unwrap();
        LocalSet::new()
            .block_on(&rt(), async {
                lua.load("fs.write(p, 'hello luna')").exec_async().await.unwrap();
                let v: String = lua.load("return fs.read(p)").eval_async().await.unwrap();
                assert_eq!(v, "hello luna");
            });
    }

    #[test]
    fn exists_returns_true_for_present_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt").to_str().unwrap().to_string();
        std::fs::write(&path, "x").unwrap();
        let lua = lua();
        let tbl = preload(&lua).unwrap();
        lua.globals().set("fs", tbl).unwrap();
        lua.globals().set("p", path).unwrap();
        let result: bool = LocalSet::new()
            .block_on(&rt(), lua.load("return fs.exists(p)").eval_async())
            .unwrap();
        assert!(result);
    }

    #[test]
    fn exists_returns_false_for_missing_file() {
        let lua = lua();
        let tbl = preload(&lua).unwrap();
        lua.globals().set("fs", tbl).unwrap();
        lua.globals().set("p", "/tmp/__luna_no_file_xyz__").unwrap();
        let result: bool = LocalSet::new()
            .block_on(&rt(), lua.load("return fs.exists(p)").eval_async())
            .unwrap();
        assert!(!result);
    }
}