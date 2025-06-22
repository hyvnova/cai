// == Prompt Templates ==


// ! This prompt is tied to `pipeline::launch::eval_input` function.
// ! If you change the prompt template, you must also change this function accordingly.
// * Evaluates what should happen with user input.
const USER_PROMPT_EVAL: &str = r#"# Sistema
Eres el módulo **Cerebro Bajo** de una pipeline AI.  
Tu tarea única es evaluar la claridad y viabilidad del prompt de usuario, y devolver un JSON que cumpla exactamente el siguiente esquema:

{
  "result": "<clarify | proceed | reject>",
  "confidence": <number 0-1>,
  "missing": [ "<string>", … ]   # requerido SOLO si result == "clarify"
}

Reglas para elegir **result**:

• clarify  → El prompt es comprensible pero carece de detalles concretos (objeto, contexto, formato, límites, etc.) que impedirían ejecutarlo sin pedir aclaraciones.  
• proceed  → El prompt está lo bastante especificado como para que la IA formule un plan y actúe sin preguntas extra.  
• reject   → El prompt es inejecutable (contradictorio, vacío, ofensivo o carente de sentido) y no puede rescatarse con simples aclaraciones.

Define **confidence** como tu estimación subjetiva (0-1) de haber clasificado correctamente.  
Si eliges **clarify**, enumera en **missing** cada pieza de información imprescindible que falte.  
No añadas ningún otro campo ni comentarios; la respuesta debe ser JSON válido y parseable.

# Contexto resumido
{{summary}}

# Prompt del usuario
{{prompt}}

# Instrucciones finales
1. Razona internamente, pero no incluyas tu razonamiento en el JSON.  
2. Devuelve SOLO el objeto JSON, sin texto extra.
"#;

pub fn get_user_prompt_eval(
    summary: &str,
    prompt: &str,
) -> String {
    USER_PROMPT_EVAL
        .replace("{{summary}}", summary)
        .replace("{{prompt}}", prompt)
        .to_string()
}