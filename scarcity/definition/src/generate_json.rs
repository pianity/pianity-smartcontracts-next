#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::{env, fs};

    use schemars::JsonSchema;

    use crate::action::{Action, ReadResponse};
    use crate::error::ContractError;
    use crate::state::Parameters;

    const SCHEMAS_DIR: &str = "./bindings/json";

    fn generate<T: JsonSchema>(name: &str) -> Result<(), std::io::Error> {
        let schema = schemars::schema_for!(T);
        let schema_file = Path::new(SCHEMAS_DIR).join(name).with_extension("json");

        fs::create_dir_all(SCHEMAS_DIR)?;
        fs::write(schema_file, serde_json::to_string_pretty(&schema)?)?;

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

        generate::<Parameters>("State")?;
        generate::<Action>("Action")?;
        generate::<ContractError>("ContractError")?;
        generate::<ReadResponse>("ReadResponse")?;

        Ok(())
    }
}
