//! Estado puro del chat: mensajes y modelo de la maqueta.
//!
//! Sin GPUI, sin IO y sin reloj: aqui vive lo testeable. `view.rs` solo lo
//! dibuja (filosofia `gentle-core`: la logica pura crece modulo a modulo y
//! los efectos se quedan en la vista).

/// Quien habla en cada fila de la conversacion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    User,
    Assistant,
}

impl Role {
    /// Etiqueta legible del rol.
    pub fn label(&self) -> &'static str {
        match self {
            Role::User => "Tú",
            Role::Assistant => "Asistente",
        }
    }
}

/// Un mensaje de la maqueta.
#[derive(Clone, Debug)]
pub struct Message {
    pub role: Role,
    pub text: String,
}

impl Message {
    /// Mensaje del usuario.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            text: text.into(),
        }
    }

    /// Mensaje del asistente.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            text: text.into(),
        }
    }
}

/// Conversacion de ejemplo que dibuja la vista.
#[derive(Clone, Debug, Default)]
pub struct ChatState {
    pub messages: Vec<Message>,
}

impl ChatState {
    /// Los cuatro mensajes fijos de la maqueta.
    pub fn seed() -> Self {
        Self {
            messages: vec![
                Message::assistant(
                    "Hola, soy una maqueta visual del chat. Todo lo que ves es \
                     estático: no hay backend conectado.",
                ),
                Message::user("¿Y los componentes de gentle-shell?"),
                Message::assistant(
                    "Card, chip, kbd, screen_header y field, copiados de \
                     shared/ui.rs y adaptados a los tokens propios. Los iconos \
                     son Tabler SVG servidos por el AssetSource.",
                ),
                Message::user("Se ve muy bien centrado."),
            ],
        }
    }

    /// Numero de mensajes.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Si no hay mensajes.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_trae_cuatro_mensajes() {
        assert_eq!(ChatState::seed().len(), 4);
    }

    #[test]
    fn seed_alterna_asistente_y_usuario() {
        let state = ChatState::seed();
        let roles: Vec<Role> = state.messages.iter().map(|m| m.role).collect();
        assert_eq!(
            roles,
            vec![Role::Assistant, Role::User, Role::Assistant, Role::User]
        );
    }

    #[test]
    fn etiquetas_de_rol() {
        assert_eq!(Role::User.label(), "Tú");
        assert_eq!(Role::Assistant.label(), "Asistente");
    }

    #[test]
    fn constructores_guardan_rol_y_texto() {
        let user = Message::user("hola");
        assert_eq!(user.role, Role::User);
        assert_eq!(user.text, "hola");

        let assistant = Message::assistant("buenas");
        assert_eq!(assistant.role, Role::Assistant);
        assert_eq!(assistant.text, "buenas");
    }

    #[test]
    fn estado_vacio_por_defecto() {
        let state = ChatState::default();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }
}
