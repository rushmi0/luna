mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use luna::LuaStdLib;

    #[test]
    fn version_reports_lua54() {
        assert_eq!(common::vm(LuaStdLib::All).version(), "Lua 5.4");
    }
}