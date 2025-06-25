pub enum Model {
    Low,
    Mid,
    Smart, // Lower context than mid, but smarter, but more expensive
    High,
}

pub fn get_model(model: Model) -> Option<String> {
    match model {
        Model::Low => Some("gpt-4.1-nano-2025-04-14".to_string()),
        Model::Mid => Some("o4-mini-2025-04-16".to_string()),
        Model::Smart => Some("gpt-4o-2024-08-06".to_string()),
        Model::High => Some("o3-2025-04-16".to_string()),
    }
}


#[macro_export]
/// Some would say this is overkill and probably dumb
/// And I say to those people: Wait until I take my meds all of you are going to dissapear HAHAHAHAHAH
macro_rules! model {
    ($variant:ident) => {
        $crate::models::get_model($crate::models::Model::$variant)
    };
}