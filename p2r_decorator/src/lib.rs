#[pyo3::pymodule]
fn p2r_decorator(_py: pyo3::Python<'_>, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    #[pyfn(m)]
    fn rust(code: String) -> String {
        p2r::p2r(&code, &mut p2r::Ctx::default()).unwrap()
    }

    Ok(())
}
