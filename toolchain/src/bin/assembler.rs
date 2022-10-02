use clap::Parser;
use peripheral_cpu::instructions::{encode_instruction, INSTRUCTION_SIZE_BYTES};
use toolchain::parsers::instruction::{parse_tokens, Token};
use toolchain::types::object::{ObjectDefinition, SymbolDefinition};

use std::fs::{read_to_string, write};
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    input_file: PathBuf,

    #[clap(short, long, value_parser, value_name = "FILE")]
    output_file: PathBuf,
}
fn build_object(tokens: Vec<Token>) -> ObjectDefinition {
    let mut symbols: Vec<SymbolDefinition> = vec![];
    let mut symbol_refs: Vec<SymbolDefinition> = vec![];
    let mut offset: u16 = 0x0;
    let mut program: Vec<[u16; 2]> = vec![];

    for token in tokens {
        match token {
            Token::Instruction(data) => {
                let encoded_instruction = encode_instruction(&data.instruction);
                program.push(encoded_instruction);
                if let Some(symbol_ref) = data.symbol_ref {
                    symbol_refs.push(SymbolDefinition {
                        name: symbol_ref.name,
                        offset: offset + symbol_ref.offset,
                    })
                }

                offset += INSTRUCTION_SIZE_BYTES;
            }
            Token::Label(data) => symbols.push(SymbolDefinition {
                name: data.name,
                offset: offset,
            }),
        }
    }

    let bytes: Vec<u8> = program
        .iter()
        .flat_map(|[b1, b2]| [u16::to_le_bytes(*b1), u16::to_le_bytes(*b2)])
        .flatten()
        .collect();

    ObjectDefinition {
        symbols,
        symbol_refs,
        program: bytes,
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let file_contents = read_to_string(args.input_file)?;
    let (_, tokens) = match parse_tokens(file_contents.as_str()) {
        Ok(tokens) => tokens,
        Err(error) => panic!("Error parsing file: {}", error),
    };
    let object_definition = build_object(tokens);
    let bytes_to_write = match postcard::to_allocvec(&object_definition) {
        Ok(bytes_to_write) => bytes_to_write,
        Err(error) => panic!("Error encoding file: {}", error),
    };

    write(args.output_file, bytes_to_write)?;
    Ok(())
}
