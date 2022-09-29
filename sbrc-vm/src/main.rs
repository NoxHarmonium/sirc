use peripheral_cpu::run_cpu;

fn main() {
    match run_cpu() {
        Ok(()) => {
            println!("CPU Done");
        }
        Err(error) => {
            panic!("CPU Error: {:#?}", error);
        }
    }
}
