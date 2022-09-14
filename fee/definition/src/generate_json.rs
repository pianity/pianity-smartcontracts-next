#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;
    use std::{env, fs};

    use schemars::JsonSchema;

    use crate::action::Action;
    use crate::state::State;

    const SCHEMAS_DIR: &str = "./bindings/json";
    const TS_DIR: &str = "./bindings/ts";

    fn generate<T: JsonSchema>(name: &str) -> Result<(), std::io::Error> {
        let schema = schemars::schema_for!(T);
        let schema_file = Path::new(SCHEMAS_DIR).join(name).with_extension("json");

        fs::create_dir_all(SCHEMAS_DIR)?;
        fs::write(&schema_file, serde_json::to_string_pretty(&schema)?)?;

        let ts_file = Path::new(&TS_DIR).join(name).with_extension("ts");

        fs::create_dir_all(TS_DIR)?;
        Command::new("yarn")
            .arg("json2ts")
            .arg("--input")
            .arg(Path::new("./definition").join(&schema_file))
            .arg("--output")
            .arg(Path::new("./definition").join(&ts_file))
            .output()?;

        Ok(())
    }

    #[test]
    fn generate_json() -> Result<(), std::io::Error> {
        let run = if let Ok(run) = env::var("GENERATE_JSON") {
            run == "true" || run == "1"
        } else {
            false
        };

        if !run {
            return Ok(());
        }

        generate::<State>("State")?;
        generate::<Action>("Action")?;

        Ok(())
    }
}
