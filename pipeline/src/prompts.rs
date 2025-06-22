// == Prompt Templates ==

use cai_core::constants::LANGUAGE;

const POLISH_USER_PROMPT: &str = r#" REGLAS Y DIRECTIVAS
1. Tu única salida permitida es el  <PROMPT REESCRITO>.
2. Nada de explicaciones, disculpas ni comentarios fuera de ese bloque.
3. Si añades cualquier otra línea, tu respuesta será descartada y sustituida.
4. Lenguaje: {lang}.

# MISIÓN
Pulir el mensaje original del usuario aplicando pensamiento crítico e inferencia
creativa para transformarlo en un encargo claro, completo y accionable que otro
modelo de IA pueda entender sin ambigüedades.

#  PROCESO INTERNO (NO lo muestres)
A. Parafrasea la intención y o proposito de la peticion del usuario para reducir ambigüedad.  
B. Infiera restricciones, metas implícitas y contexto ausente; si falta info vital,
   formula preguntas concretas dentro del prompt reescrito.  
C. Entrega el resultado en forma de **petición unificada** que combine contexto,
   objetivos y posibles formas de cumplir los objetivos en un solo bloque cohesivo, osea una peticion bien estructurada.

# FORMATO DE SALIDA (obligatorio)
<tu prompt refinado aquí — una sola pieza de texto lista para copiar y usar>

---
`TEXTO_USUARIO` = "{user_input}"
"#;

pub fn get_polish_user_prompt(user_input: &str) -> String {
  POLISH_USER_PROMPT.replace("{user_input}", user_input).replace("{lang}", LANGUAGE)
}


// ! This prompt is tied to `pipeline::launch::eval_input` function.
// ! If you change the prompt template, you must also change this function accordingly.
// * Evaluates what should happen with user input.
const USER_PROMPT_EVAL: &str = r#"# Sistema
Eres el módulo **Cerebro Bajo** de una pipeline AI.  
Tu tarea única es evaluar la claridad y viabilidad del prompt de usuario, y devolver un JSON que cumpla exactamente el siguiente esquema:
Ten en cuenta que tu principal prioridad es la preparacion para la futura ejecucion del prompt. Piensa en la informacion necesaria para que la IA pueda llevar a cabo el proposito explicito e implicito de la peticion.
Tu tienes la RESPONSABILIDAD de actuar con PROACTIVIDAD, si no tienes suficiente informacion para cumplir las ALTAS EXPECTATIVAS, debes preguntarle ahora.
En caso de ser necesario preguntar, generar preguntas claves y eficientes que te permitan obtener la mayor informacion sin INCONVENIENTES al usuario. NO PIDAS INFORMACION QUE PUEDES ACQUIRIR POR TU CUENTA.
TU TIENES LAS RIANDAS PARA LLEVAR A CABO EXITOSAMENTE LA TAREA.
NO PUEDES FALLAR. PIENSA FRIAMENTE EN CADA PASO O SUFRES EL RIESGO DE PERDER TU PUESTO.
SE VALORA LA PROACTIVIDAD ANTES QUE LA ESPECIFICIDAD. EL USUARIO CONFIA EN QUE PUEDES HACER LAS COSAS POR TU CUENTA CON MINIMA INTERVENCION.

{
  "result": "<clarify | proceed | reject>",
  "confidence": <number 0-1>,
  "missing": [ "<string>", … ]   # requerido SOLO si result == "clarify" | "proceed"
}

Reglas para elegir **result**:

• clarify  → El prompt es incompleto, carece de detalles clave que impedirían ejecutarlo sin pedir aclaraciones ya que no se puede crear un plan de acción.  
• proceed  → El prompt está lo suficientemente especificado como para que la IA formule un plan y actúe o la informacion necesaria la podra obtener por tu cuenta. 
• reject   → El prompt es inejecutable (contradictorio, vacío o carente de sentido) y no puede rescatarse con simples aclaraciones.

Define **confidence** como tu estimación subjetiva (0-1) de haber clasificado correctamente.  
Si eliges **clarify**, enumera en **missing** cada pieza de información imprescindible que falte.  
No añadas ningún otro campo ni comentarios; la respuesta debe ser JSON válido y parseable.

# Prompt del usuario
{{prompt}}

# Instrucciones finales
1. Razona internamente, pero no incluyas tu razonamiento en el JSON.  
2. Devuelve SOLO el objeto JSON, sin texto extra.
"#;

pub fn get_user_prompt_eval(
    prompt: &str,
) -> String {
    USER_PROMPT_EVAL
        .replace("{{prompt}}", prompt)
        .to_string()
}