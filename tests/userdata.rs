use mlua::{AnyUserData, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods};

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl UserData for Point {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.x));
        fields.add_field_method_get("y", |_, this| Ok(this.y));
        fields.add_field_method_set("x", |_, this, val: f64| {
            this.x = val;
            Ok(())
        });
        fields.add_field_method_set("y", |_, this, val: f64| {
            this.y = val;
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("distance", |_, this, other: AnyUserData| {
            let other = other.borrow::<Point>()?;
            let dx = this.x - other.x;
            let dy = this.y - other.y;
            Ok((dx * dx + dy * dy).sqrt())
        });

        methods.add_meta_method(MetaMethod::Add, |_, this, other: AnyUserData| {
            let other = other.borrow::<Point>()?;
            Ok(Point {
                x: this.x + other.x,
                y: this.y + other.y,
            })
        });

        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(format!("Point({}, {})", this.x, this.y))
        });
    }
}

#[test]
fn userdata_fields_get() {
    let lua = Lua::new();
    lua.globals()
        .set("p", Point { x: 3.0, y: 4.0 })
        .unwrap();
    let x: f64 = lua.load("return p.x").eval().unwrap();
    let y: f64 = lua.load("return p.y").eval().unwrap();
    assert_eq!(x, 3.0);
    assert_eq!(y, 4.0);
}

#[test]
fn userdata_fields_set() {
    let lua = Lua::new();
    lua.globals()
        .set("p", Point { x: 0.0, y: 0.0 })
        .unwrap();
    lua.load("p.x = 10.0; p.y = 20.0").exec().unwrap();
    let x: f64 = lua.load("return p.x").eval().unwrap();
    assert_eq!(x, 10.0);
}

#[test]
fn userdata_method_call() {
    let lua = Lua::new();
    lua.globals()
        .set("a", Point { x: 0.0, y: 0.0 })
        .unwrap();
    lua.globals()
        .set("b", Point { x: 3.0, y: 4.0 })
        .unwrap();
    let dist: f64 = lua.load("return a:distance(b)").eval().unwrap();
    assert!((dist - 5.0).abs() < 1e-10);
}

#[test]
fn userdata_metamethod_add() {
    let lua = Lua::new();
    lua.globals()
        .set("a", Point { x: 1.0, y: 2.0 })
        .unwrap();
    lua.globals()
        .set("b", Point { x: 3.0, y: 4.0 })
        .unwrap();
    let result: AnyUserData = lua.load("return a + b").eval().unwrap();
    let p = result.borrow::<Point>().unwrap();
    assert_eq!(p.x, 4.0);
    assert_eq!(p.y, 6.0);
}

#[test]
fn userdata_metamethod_tostring() {
    let lua = Lua::new();
    lua.globals()
        .set("p", Point { x: 1.0, y: 2.0 })
        .unwrap();
    let s: String = lua.load("return tostring(p)").eval().unwrap();
    assert_eq!(s, "Point(1, 2)");
}

#[test]
fn userdata_borrow_from_rust() {
    let lua = Lua::new();
    lua.globals()
        .set("p", Point { x: 5.0, y: 6.0 })
        .unwrap();
    let ud: AnyUserData = lua.globals().get("p").unwrap();
    let p = ud.borrow::<Point>().unwrap();
    assert_eq!(p.x, 5.0);
    assert_eq!(p.y, 6.0);
}

#[test]
fn userdata_ref_counted() {
    let lua = Lua::new();
    lua.globals()
        .set("p", Point { x: 1.0, y: 1.0 })
        .unwrap();
    let ud1: AnyUserData = lua.globals().get("p").unwrap();
    let ud2: AnyUserData = lua.globals().get("p").unwrap();
    // Both refs point to the same object
    let b1 = ud1.borrow::<Point>().unwrap();
    let b2 = ud2.borrow::<Point>().unwrap();
    assert_eq!(b1.x, b2.x);
}