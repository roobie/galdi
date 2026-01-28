use jaq_core::{Ctx, RcIter, load};
use jaq_json::Val;
use serde_json::Value;

pub struct Jaq {}

impl Jaq {
    pub fn apply_jaq_filter(jq: &str, input: &Value) -> String {
        use load::{Arena, File, Loader};
        let program = File { code: jq, path: () };

        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();

        // parse the filter
        let modules = loader.load(&arena, program).unwrap();

        // compile the filter
        let filter = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .unwrap();

        let inputs = RcIter::new(core::iter::empty());

        // iterator over the output values
        let mut out = filter.run((Ctx::new([], &inputs), Val::from(input.clone())));
        return match out.next() {
            Some(Ok(v)) => v.to_string(),
            Some(Err(e)) => format!("Jaq filter error: {}", e),
            None => String::new(),
        };
    }
}
