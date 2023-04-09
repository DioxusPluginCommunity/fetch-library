use std::{collections::HashMap, io::Read};

use mlua::prelude::*;
use reqwest::header::HeaderMap;

// fn hello(_: &Lua, name: String) -> LuaResult<()> {
//     println!("hello, {}!", name);
//     Ok(())
// }

#[derive(Clone, Default)]
struct Headers(HashMap<String, String>);

#[derive(Clone)]
struct Response {
    status: u16,
    data: String,
    headers: Headers,
}

impl LuaUserData for Response {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("status", |_, this| Ok(this.status));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("is_success", |_, this, ()| Ok(this.status == 200));
    }
}

#[derive(Clone)]
struct Request {
    method: String,
    url: String,
    data: RequestData,
    headers: Headers,
}

#[derive(Clone)]
enum RequestData {
    None,
    Form(HashMap<String, String>),
    Multipart(MutliForm),
}

#[derive(Clone, Default)]
struct MutliForm {
    text: HashMap<String, String>,
    file: HashMap<String, String>,
}

impl LuaUserData for Request {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("header", |_, this, (key, value): (String, String)| {
            this.headers.0.insert(key, value);
            Ok(())
        });

        methods.add_method_mut("form", |_, this, form: HashMap<String, String>| {
            this.data = RequestData::Form(form);
            Ok(())
        });

        methods.add_method_mut("multi_text", |_, this, (key, value): (String, String)| {
            let data_type = this.data.clone();
            if let RequestData::Multipart(mut v) = data_type {
                v.text.insert(key, value);
                this.data = RequestData::Multipart(v);
            } else {
                this.data = RequestData::Multipart(MutliForm::default());
            }
            Ok(())
        });

        methods.add_method_mut("multi_file", |_, this, (key, value): (String, String)| {
            let data_type = this.data.clone();
            if let RequestData::Multipart(mut v) = data_type {
                v.file.insert(key, value);
                this.data = RequestData::Multipart(v);
            } else {
                this.data = RequestData::Multipart(MutliForm::default());
            }
            Ok(())
        });

        methods.add_method("send", |_, this, ()| {
            let client = reqwest::blocking::Client::new();

            let request = match this.method.as_str() {
                "GET" => client.get(&this.url),
                "POST" => client.post(&this.url),
                "PUT" => client.put(&this.url),
                _ => {
                    panic!("Unsopprted request method");
                }
            };

            let headers: HeaderMap = (&this.headers.0).try_into().unwrap();
            let request = request.headers(headers);

            let request = match &this.data {
                RequestData::None => request,
                RequestData::Form(f) => request.form(&f),
                RequestData::Multipart(f) => {
                    let mut request = request;
                    let text = f.text.clone();
                    let file = f.file.clone();
                    let mut form = reqwest::blocking::multipart::Form::new();
                    for i in text {
                        form = form.text(i.0, i.1);
                    }
                    for i in file {
                        let file_path = std::path::PathBuf::from(&i.0);
                        let file = std::fs::File::open(&file_path);
                        if let Ok(mut f) = file {
                            let mut buf: Vec<u8> = Vec::new();
                            let _ = f.read_to_end(&mut buf);
                            let part = reqwest::blocking::multipart::Part::bytes(buf).file_name(
                                file_path.file_name().unwrap().to_str().unwrap().to_string(),
                            );
                            form = form.part(i.0, part);
                        }
                    }

                    request.multipart(form)
                }
            };

            let response = request.send().unwrap();

            let mut headers = HashMap::new();
            for i in response.headers().iter() {
                headers.insert(i.0.to_string(), i.1.to_str().unwrap().to_string());
            }

            let result = Response {
                status: response.status().as_u16(),
                data: response.text().unwrap(),
                headers: Headers(headers),
            };

            Ok(result)
        });
    }
}

fn get(_lua: &Lua, url: String) -> LuaResult<Request> {
    Ok(Request {
        method: "GET".into(),
        url: url.into(),
        data: RequestData::None,
        headers: Headers(HashMap::new()),
    })
}

fn post(_lua: &Lua, url: String) -> LuaResult<Request> {
    Ok(Request {
        method: "POST".into(),
        url: url.into(),
        data: RequestData::None,
        headers: Headers(HashMap::new()),
    })
}

#[mlua::lua_module]
fn my_module(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("get", lua.create_function(get)?)?;
    exports.set("post", lua.create_function(post)?)?;
    Ok(exports)
}
