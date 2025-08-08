pub enum Model {
    Nano,   // ultra-cheap, basic stuff
    Mini,   // fast + good for mid-tier reasoning
    Full,   // full GPT-5 power
    Max,    // same as Full but with maximum context / heavy reasoning
}

pub fn get_model(model: Model) -> Option<String> {
    match model {
        Model::Nano => Some("gpt-5-nano".to_string()),    // cheap/trivial
        Model::Mini => Some("gpt-5-mini".to_string()),    // balanced
        Model::Full => Some("gpt-5".to_string()),         // full reasoning
        Model::Max  => Some("gpt-5".to_string()),         // same model, but you’d set max context/tooling in the API call
    }
}

#[macro_export]
macro_rules! model {
    ($variant:ident) => {
        $crate::models::get_model($crate::models::Model::$variant)
    };
}
