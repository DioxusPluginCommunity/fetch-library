use mlua::prelude::*;

// fn hello(_: &Lua, name: String) -> LuaResult<()> {
//     println!("hello, {}!", name);
//     Ok(())
// }

#[derive(Clone)]
struct Response {
    status: u16,
    text: String,
}

impl LuaUserData for Response {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("status", |_, this| Ok(this.status));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("is_success", |_, this, ()| Ok(this.status == 200));
        methods.add_method("text", |_, this, ()| Ok(this.text.clone()))
    }
}

fn get(_: &Lua, url: String) -> LuaResult<Response> {
    let res = reqwest::blocking::get(url);
    if let Err(_e) = res {
        return Ok(Response {
            status: 400,
            text: String::new(),
        });
    }
    let res = res.unwrap();
    let status = res.status().as_u16();
    let text = res.text().unwrap();
    let response = Response { status, text };
    Ok(response)
}

#[mlua::lua_module]
fn my_module(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    Ok(exports)
}
