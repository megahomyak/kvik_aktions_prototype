use std::{cell::RefCell, rc::Rc};

include!("gui.slint");

struct Action<Callback> {
    name: &'static str,
    shortcut: String,
    callback: Callback,
}

impl<Callback: Fn() -> bool> Action<Callback> {
    pub fn new(name: &'static str, callback: Callback) -> Self {
        Self {
            name,
            shortcut: name.chars().filter(|char| char.is_uppercase()).collect(),
            callback,
        }
    }
}

fn make_message_action(ui: &KvikAktions, text: &'static str) -> Action<Box<dyn Fn() -> bool>> {
    Action::new(text, {
        let ui = ui.as_weak().unwrap();
        Box::new(move || {
            ui.invoke_show_message(text.into());
            true
        })
    })
}

struct Stack<'ui> {
    ui: &'ui KvikAktions,
    contents: Vec<i32>,
}

impl<'ui> Stack<'ui> {
    pub fn push(&mut self, value: i32) {
        self.contents.push(value);
    }

    pub fn pop(&mut self, line_number: i32, command_number: i32) -> i32 {
        match self.contents.pop() {
            Some(value) => value,
            None => {
                self.ui.invoke_show_message(
                    format!("Stack is empty at line {line_number}, command {command_number}!")
                        .into(),
                );
                0
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn main() {
    let ui = KvikAktions::new();
    let all_actions = Rc::new(RefCell::new(vec![
        make_message_action(&ui, "Hello, World!"),
        make_message_action(&ui, "Another Action"),
        make_message_action(&ui, "Apple Pie"),
        make_message_action(&ui, "CAPITALIZED"),
    ]));
    let matches = Rc::new(slint::VecModel::<ListItem>::from(vec![]));
    matches.set_vec(
        all_actions
            .borrow()
            .iter()
            .map(|action| ListItem {
                text: action.name.into(),
            })
            .collect::<Vec<_>>(),
    );
    ui.on_update_matches({
        let matches = matches.clone();
        let ui = ui.as_weak().unwrap();
        let all_actions = all_actions.clone();
        move |query| {
            let query = query
                .chars()
                .filter(|char| char.is_alphabetic())
                .collect::<String>()
                .to_uppercase();
            let mut matching_actions = vec![];
            let all_actions = &*all_actions.borrow_mut();
            for action in all_actions {
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

    ui.on_run_script({
        let all_actions = all_actions.clone();
        let ui = ui.as_weak().unwrap();
        move |script: slint::SharedString| {
            let mut stack = Stack {
                contents: vec![],
                ui: &ui,
            };
            let mut blocks_amount: usize = 0;
            let mut skip_current_block = false;
            for (line, line_number) in script.lines().zip(1..) {
                let line = match line.split_once('#') {
                    None => line,
                    Some((command, _commentary)) => command,
                };
                let shortcut = line
                    .chars()
                    .filter(|char| char.is_uppercase())
                    .collect::<String>();
                let is_action = !shortcut.is_empty();
                if is_action {
                    if !skip_current_block {
                        if let Some(action) = all_actions
                            .borrow_mut()
                            .iter_mut()
                            .find(|action| action.shortcut == shortcut)
                        {
                            stack.push((action.callback)().into());
                        } else {
                            ui.invoke_show_message(
                                format!(
                                    "Command with shortcut \"{shortcut}\" (found at line \
                                    {line_number}) not found!"
                                )
                                .into(),
                            );
                        }
                    }
                } else {
                    let mut line_iterator = line.split_whitespace();
                    if skip_current_block {
                        for command in line_iterator.by_ref() {
                            if command == "." {
                                skip_current_block = false;
                                break;
                            }
                        }
                    }
                    for (command, command_number) in line_iterator.zip(1..) {
                        if let Ok(number) = command.parse::<i32>() {
                            stack.push(number);
                        } else {
                            match command {
                                "." => {
                                    if blocks_amount == 0 {
                                        stack.ui.invoke_show_message(
                                            format!(
                                                "Unexpected end of block at line {line_number}, \
                                            command {command_number}!"
                                            )
                                            .into(),
                                        );
                                    } else {
                                        blocks_amount -= 1;
                                    }
                                }
                                "=" => {
                                    if stack.pop(line_number, command_number)
                                        == stack.pop(line_number, command_number)
                                    {
                                        blocks_amount += 1;
                                    } else {
                                        skip_current_block = true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
        }
    });

    ui.run();
}
