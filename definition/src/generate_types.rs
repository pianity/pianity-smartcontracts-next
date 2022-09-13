#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::{env, fs};

    use schemars::JsonSchema;

    use crate::action::Action;
    use crate::state::State;

    const SCHEMAS_DIR: &str = "./bindings/json";

    fn generate<T: JsonSchema>(name: &str) -> Result<(), std::io::Error> {
        let schema = schemars::schema_for!(T);
        let schema_file = Path::new(SCHEMAS_DIR).join(name).with_extension("json");

        fs::create_dir_all(SCHEMAS_DIR)?;
        fs::write(&schema_file, serde_json::to_string_pretty(&schema)?)?;

        Ok(())
    }

    #[test]
    fn run_generate() -> Result<(), std::io::Error> {
        let run = if let Ok(run) = env::var("RUN_GENERATE") {
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
