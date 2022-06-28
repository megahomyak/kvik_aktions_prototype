use std::rc::Rc;

include!("gui.slint");

struct Action<Callback> {
    name: &'static str,
    shortcut: String,
    callback: Callback,
}

impl<Callback: Fn()> Action<Callback> {
    pub fn new(name: &'static str, callback: Callback) -> Self {
        Self {
            name,
            shortcut: name.chars().filter(|char| char.is_uppercase()).collect(),
            callback,
        }
    }
}

fn make_message_action(ui: &KvikAktions, text: &'static str) -> Action<Box<dyn Fn()>> {
    Action::new(text, {
        let ui = ui.as_weak().unwrap();
        Box::new(move || {
            ui.invoke_show_message(text.into());
        })
    })
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
    matches.set_vec(
        all_actions
            .iter()
            .map(|action| ListItem {
                text: action.name.into(),
            })
            .collect::<Vec<_>>(),
    );
    ui.on_update_matches({
        let matches = matches.clone();
        let ui = ui.as_weak().unwrap();
        move |query| {
            let query = query
                .chars()
                .filter(|char| char.is_alphabetic())
                .collect::<String>()
                .to_uppercase();
            let mut matching_actions = vec![];
            for action in &all_actions {
                if action.shortcut == query.to_uppercase() {
                    (action.callback)();
                    matches.set_vec(
                        all_actions
                            .iter()
                            .map(|action| ListItem {
                                text: action.name.into(),
                            })
                            .collect::<Vec<_>>(),
                    );
                    ui.invoke_reset_query();
                    return;
                }
                if action.shortcut.starts_with(&query.to_uppercase()) {
                    matching_actions.push(action);
                }
            }
            matches.set_vec(
                matching_actions
                    .iter()
                    .map(|action| ListItem {
                        text: action.name.into(),
                    })
                    .collect::<Vec<_>>(),
            );
        }
    });
    ui.set_matches(matches.into());

    ui.on_run_script(|script: slint::SharedString| {
        for line in script.lines() {
            let line = match line.split_once("#") {
                None => &line,
                Some((command, _commentary)) => command,
            };
            let shortcut = line.chars().filter(|char| char.is_uppercase()).collect::<String>();
            if shortcut.len() == 0 {

            } else {
                for action in &all_actions {
                    if action.shortcut == shortcut {
                        (action.callback)();
                        break;
                    }
                }
            }
        }
    });

    ui.run();
}
