use luna::{LocalValue, LuaOption, LuaStdLib, LuaVersion, LunaVM};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- start() ---
    // Create the VM with a struct literal, then call .start().
    let vm = LunaVM {
        config: LuaOption {
            stdlib: LuaStdLib::All,
            version: LuaVersion::Lua54,
        },
    }
    .start()?;

    println!("Lua version: {}", vm.version());

    // --- run() ---
    // Evaluates an expression and returns the result as LocalValue.
    // State set here persists for the lifetime of the VM.
    let result = vm.run("return 2 + 3".to_string())?;
    println!("2 + 3 = {:?}", result); // Integer(5)

    vm.run("answer = 42".to_string())?;
    let answer = vm.run("return answer".to_string())?;
    println!("answer = {:?}", answer); // Integer(42)

    // All LocalValue variants
    let _nil = vm.run("return nil".to_string())?;
    let _boolean = vm.run("return true".to_string())?;
    let _integer = vm.run("return 100".to_string())?;
    let _float = vm.run("return 3.14".to_string())?;
    let _string = vm.run(r#"return "hello""#.to_string())?;

    // Tables and functions collapse to Nil (not yet bridged).
    let table = vm.run("return {}".to_string())?;
    assert_eq!(table, LocalValue::Nil);

    // --- exec() ---
    // Executes a statement without returning a value.
    // Returns Ok(true) on success, Err(...) on failure.
    // `local` scopes a variable to its chunk; globals persist across calls.
    vm.exec("a = 2")?;
    vm.exec("print(a + 10)")?; // prints 12

    let ok = vm.exec("x = 1")?;
    assert!(ok);

    let err = vm.exec("error('boom')");
    println!("exec error: {}", err.unwrap_err());

    // State set by exec is visible to run and vice versa.
    vm.exec("counter = 0")?;
    vm.exec("counter = counter + 1")?;
    let count = vm.run("return counter".to_string())?;
    println!("counter = {:?}", count); // Integer(1)

    // --- set_global() / get_global() ---
    // Inject a value from Rust into the Lua environment.
    vm.set_global("threshold".to_string(), LocalValue::Integer(10))?;
    let result = vm.run("return threshold * 2".to_string())?;
    println!("threshold * 2 = {:?}", result); // Integer(20)

    // Read a global written by a Lua script.
    vm.run("computed = 7 * 6".to_string())?;
    let computed = vm.get_global("computed".to_string())?;
    println!("computed = {:?}", computed); // Integer(42)

    // All LocalValue types can be set.
    vm.set_global("flag".to_string(), LocalValue::Boolean(true))?;
    vm.set_global("ratio".to_string(), LocalValue::Number(0.5))?;
    vm.set_global("label".to_string(), LocalValue::LuaString("ok".to_string()))?;
    vm.set_global("empty".to_string(), LocalValue::Nil)?;

    // --- run_file() ---
    // Write a temporary Lua script and execute it.
    let path = std::env::temp_dir().join("luna_example.lua");
    std::fs::write(&path, "file_result = 100 + 1\n")?;
    vm.run_file(path.to_str().unwrap().to_string())?;
    let file_result = vm.get_global("file_result".to_string())?;
    println!("file_result = {:?}", file_result); // Integer(101)

    let missing = vm.run_file("/tmp/does_not_exist.lua".to_string());
    println!("missing file error: {}", missing.unwrap_err());

    println!("done");
    Ok(())
}
