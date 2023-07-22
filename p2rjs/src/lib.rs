use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Res {
    // cheap impl of tagged unioni
    t: String,

    code: Option<String>,

    file: Option<String>,
    line: Option<u32>,

    parse_error: Option<String>,
}

#[wasm_bindgen]
impl Res {
    #[wasm_bindgen(getter)]
    pub fn res_t(&self) -> String {
        self.t.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn code(&self) -> Option<String> {
        self.code.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn file(&self) -> Option<String> {
        self.file.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    #[wasm_bindgen(getter)]
    pub fn parse_error(&self) -> Option<String> {
        self.parse_error.clone()
    }
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn p2r(content: &str) -> Res {
    let mut ctx = p2r::Ctx::default();
    let res = p2r::p2r(content, &mut ctx);

    match res {
        Ok(code) => Res {
            t: "ok".to_string(),
            code: Some(code),
            file: None,
            line: None,
            parse_error: None,
        },
        Err(p2r::ParseError::TranspileError(e)) => Res {
            t: "TranspileError".to_string(),
            code: None,
            file: Some(e.file.to_string()),
            line: Some(e.line),
            parse_error: None,
        },
        Err(p2r::ParseError::ParseError(e)) => Res {
            t: "ParseError".to_string(),
            code: None,
            file: None,
            line: None,
            parse_error: Some(e.to_string()),
        },
    }
}
