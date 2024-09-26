pub mod cairo_run;
pub mod error;
use std::path::PathBuf;

pub use crate::cairo_run::FuncArg;
use crate::cairo_run::{cairo_run_program, Cairo1RunConfig};
pub use cairo_vm::types::layout_name::LayoutName;
pub use cairo_vm::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::{runners::cairo_runner::CairoRunner, vm_core::VirtualMachine},
    Felt252,
};
use error::Error;

pub fn get_cairo_pie(
    program_file: PathBuf,
    output_file: PathBuf,
    layout: LayoutName,
    args: FuncArg,
) -> Result<Option<String>, Error> {
    let cairo_run_config = Cairo1RunConfig {
        proof_mode: false,
        serialize_output: true,
        relocate_mem: false,
        layout,
        trace_enabled: false,
        args: &[args],
        finalize_builtins: true,
        append_return_values: false,
    };

    // Try to parse the file as a sierra program
    let file = std::fs::read(&program_file)?;
    let sierra_program = match serde_json::from_slice(&file) {
        Ok(program) => program,
        Err(_) => {
            println!("Failed to parse the file as a sierra program");
            return Err(Error::SierraCompilation(
                "Failed to parse the file as a sierra program".to_string(),
            ));
        }
    };
    let (runner, _, serialized_output) = cairo_run_program(&sierra_program, cairo_run_config)?;
    runner.get_cairo_pie()?.write_zip_file(&output_file)?;

    Ok(serialized_output)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::{get_cairo_pie, FuncArg};
    use cairo_vm::types::layout_name::LayoutName;
    use itertools::Itertools;
    use starknet_types_core::felt::Felt;
    use std::path::PathBuf;
    #[test]
    fn test_get_cairo_pie() -> Result<(), Error> {
        let input = vec![
            Felt::from(1),
            Felt::from_dec_str(
                "1084568281184221360887085980818130019116060769753707796384172133640093947392",
            )
            .unwrap(),
            Felt::from_dec_str(
                "617075754465154585683856897856256838130216341506379215893724690153393808813",
            )
            .unwrap(),
            Felt::from(4),
            Felt::from(1),
            Felt::from_dec_str(
                "1962399278914746334808042087624794244340195160841430388580769389462301739649",
            )
            .unwrap(),
            Felt::from_dec_str(
                "946638316592298107720053446348402985413061731752482859793467974131030053837",
            )
            .unwrap(),
            Felt::from(0),
            Felt::from(0),
            Felt::from(0),
            Felt::from(193823),
            Felt::from(0),
            Felt::from(0),
        ];
        let filename = PathBuf::from("batcher.json");
        let cairo_pie_output = PathBuf::from("batcher.zip");
        let layout = LayoutName::recursive;
        let args = FuncArg::Array(input);

        match get_cairo_pie(filename, cairo_pie_output, layout, args) {
            Err(Error::Cli(err)) => err.exit(),
            Ok(output) => {
                if let Some(output_string) = output {
                    println!("Program Output : {}", output_string);
                }
                Ok(())
            }
            Err(Error::RunPanic(panic_data)) => {
                if !panic_data.is_empty() {
                    let panic_data_string_list = panic_data
                        .iter()
                        .map(|m| {
                            // Try to parse to utf8 string
                            let msg = String::from_utf8(m.to_bytes_be().to_vec());
                            if let Ok(msg) = msg {
                                format!("{} ('{}')", m, msg)
                            } else {
                                m.to_string()
                            }
                        })
                        .join(", ");
                    println!("Run panicked with: [{}]", panic_data_string_list);
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
