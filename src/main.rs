use std::rc::Rc;

include!("gui.slint");

struct Action<Callback> {
    name: &'static str,
    shortcut: String,
    callback: Callback,
}

impl<Callback: FnMut()> Action<Callback> {
    pub fn new(name: &'static str, callback: Callback) -> Self {
        Self {
            name,
            shortcut: name.chars().filter(|char| char.is_uppercase()).collect(),
            callback,
        }
    }
}

fn make_message_action(ui: &KvikAktions, text: &'static str) -> Action<Box<dyn FnMut()>> {
    Action::new(text, {
        let ui = ui.as_weak().unwrap();
        Box::new(move || {
            ui.invoke_show_message(text.into());
        })
    })
}

fn set_matches<'action, Callback: 'action>(
    matches: &Rc<slint::VecModel<ListItem>>,
    matching_actions: impl Iterator<Item = &'action mut Action<Callback>>,
) {
    matches.set_vec(
        matching_actions
            .map(|action| ListItem {
                text: action.name.into(),
            })
            .collect::<Vec<_>>(),
    );
}

fn main() {
    let ui = KvikAktions::new();
    let mut all_actions = vec![
        make_message_action(&ui, "Hello, World!"),
        make_message_action(&ui, "Another Action"),
        make_message_action(&ui, "Apple Pie"),
        make_message_action(&ui, "CAPITALIZED"),
    ];
    let matches = Rc::new(slint::VecModel::<ListItem>::from(vec![]));
    set_matches(&matches, all_actions.iter_mut());
    ui.on_update_matches({
        let matches = matches.clone();
        let ui = ui.as_weak().unwrap();
        move |query| {
            let query = query.to_uppercase();
            let mut matching_actions = vec![];
            for action in &mut all_actions {
                if action.shortcut == query {
                    (action.callback)();
                    set_matches(&matches, all_actions.iter_mut());
                    ui.invoke_reset_query();
                    return;
                }
                if action.shortcut.starts_with(&query) {
                    matching_actions.push(action);
                }
            }
            set_matches(&matches, matching_actions.into_iter());
        }
    });
    ui.set_matches(matches.into());
    ui.run();
}
