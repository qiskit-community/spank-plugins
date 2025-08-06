use pyo3_stub_gen::Result;
use qrmi::pyext::stub_info;

fn main() -> Result<()> {
    let stub = stub_info()?;
    stub.generate()?;
    Ok(())
}
