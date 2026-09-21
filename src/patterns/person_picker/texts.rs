use super::model::PersonPresence;

/// Framework-owned copy. Pass it as a `Signal` so a language that arrives
/// after mount propagates instead of freezing (Office op-e6dsi).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonPickerTexts {
    /// Search box placeholder.
    pub search_placeholder: String,
    /// Search box accessible name.
    pub search_label: String,
    /// The search box's x.
    pub clear_search: String,
    /// The listbox's accessible name.
    pub roster_label: String,
    /// Shown when the roster itself is empty.
    pub empty_roster: String,
    /// Shown when the search matches nobody.
    pub no_matches: String,
    /// Leads the selected badge.
    pub selected_prefix: String,
    /// The selected badge's x.
    pub clear_selection: String,
    /// Collapse toggle, while collapsed.
    pub show: String,
    /// Collapse toggle, while expanded.
    pub hide: String,
    /// Presence word.
    pub available: String,
    /// Presence word.
    pub busy: String,
    /// Presence word.
    pub away: String,
    /// Presence word.
    pub offline: String,
}

impl Default for PersonPickerTexts {
    fn default() -> Self {
        Self {
            search_placeholder: "Find a person".to_owned(),
            search_label: "Search people".to_owned(),
            clear_search: "Clear search".to_owned(),
            roster_label: "People".to_owned(),
            empty_roster: "No one is on this roster.".to_owned(),
            no_matches: "No one matches your search.".to_owned(),
            selected_prefix: "Selected:".to_owned(),
            clear_selection: "Clear selection".to_owned(),
            show: "Show".to_owned(),
            hide: "Hide".to_owned(),
            available: "Available".to_owned(),
            busy: "Busy".to_owned(),
            away: "Away".to_owned(),
            offline: "Offline".to_owned(),
        }
    }
}

impl PersonPickerTexts {
    /// Spanish.
    pub fn es() -> Self {
        Self {
            search_placeholder: "Buscar una persona".to_owned(),
            search_label: "Buscar personas".to_owned(),
            clear_search: "Borrar búsqueda".to_owned(),
            roster_label: "Personas".to_owned(),
            empty_roster: "No hay nadie en esta lista.".to_owned(),
            no_matches: "Nadie coincide con su búsqueda.".to_owned(),
            selected_prefix: "Seleccionado:".to_owned(),
            clear_selection: "Borrar selección".to_owned(),
            show: "Mostrar".to_owned(),
            hide: "Ocultar".to_owned(),
            available: "Disponible".to_owned(),
            busy: "Ocupado".to_owned(),
            away: "Ausente".to_owned(),
            offline: "Desconectado".to_owned(),
        }
    }

    /// The word shown beside the presence dot.
    pub fn presence(&self, presence: PersonPresence) -> &str {
        match presence {
            PersonPresence::Available => &self.available,
            PersonPresence::Busy => &self.busy,
            PersonPresence::Away => &self.away,
            PersonPresence::Offline => &self.offline,
        }
    }
}
