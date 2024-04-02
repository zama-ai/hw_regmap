use std::collections::HashMap;

use tera::{from_value, to_value, Tera, Value};

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Register {
    name: String,
    dtf: usize,
}

fn main() {
    // Create a new Tera instance and add a template from a string
    let mut tera = Tera::new("templates/**/*.sv").unwrap();
    tera.add_raw_template("hello", "Hello, {{ name }}!")
        .unwrap();

    // Prepare the context with some data
    let mut context = tera::Context::new();
    context.insert("name", "World");
    let var_v = vec![
        Register {
            name: "Albert".to_string(),
            dtf: 4,
        },
        Register {
            name: "Robert".to_string(),
            dtf: 40,
        },
        Register {
            name: "Timmy".to_string(),
            dtf: 8,
        },
    ];
    context.insert("vec", &var_v);

    let mut hash = HashMap::new();

    hash.insert("RegA".to_string(), var_v[0].clone());
    hash.insert("RegB".to_string(), var_v[1].clone());
    hash.insert("RegC".to_string(), var_v[2].clone());

    context.insert("hash", &hash);

    // Adding some function
    fn make_gen_io(register: HashMap<String, Register>) -> impl tera::Function {
        Box::new(
            move |args: &HashMap<String, Value>| -> tera::Result<Value> {
                match args.get("name") {
                    Some(val) => match from_value::<String>(val.clone()) {
                        Ok(v) => Ok(to_value(register.get(&v).unwrap().name.clone()).unwrap()),
                        Err(_) => Err("oops".into()),
                    },
                    None => Err("oops".into()),
                }
            },
        )
    }

    tera.register_function("gen_io", make_gen_io(hash.clone()));

    // Render the template with the given context
    let rendered = tera.render("hello", &context).unwrap();
    println!("=> {rendered}");

    let rendered = tera.render("module.sv", &context).unwrap();
    println!("=> {rendered}");
}
