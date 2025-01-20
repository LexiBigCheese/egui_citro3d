#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImeStage {
    Nothing,
    SelectAllDown,
    SelectAllUp,
    BackSpaceDown,
    BackSpaceUp,
    PutText,
    EscapeDown,
    EscapeUp,
}

impl ImeStage {
    pub(crate) const START: ImeStage = ImeStage::SelectAllDown;
    pub(crate) const CANCEL: ImeStage = ImeStage::EscapeDown;
    pub(crate) fn next(self) -> Self {
        use ImeStage::*;
        match self {
            Nothing => Nothing,
            SelectAllDown => SelectAllUp,
            SelectAllUp => BackSpaceDown,
            BackSpaceDown => BackSpaceUp,
            BackSpaceUp => PutText,
            PutText => EscapeDown,
            EscapeDown => EscapeUp,
            EscapeUp => Nothing,
        }
    }
    pub(crate) fn add_event(self, events: &mut Vec<egui::Event>) -> bool {
        use ImeStage::*;
        match self {
            Nothing => false,
            SelectAllDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: true,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            SelectAllUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: false,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            BackSpaceDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            BackSpaceUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            PutText => true,
            EscapeDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            EscapeUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
        }
    }
}

pub(crate) fn ime_part_b(ime: &mut Option<egui::output::IMEOutput>, ime_stage: &ImeStage, current_text_value: &mut Option<String>, current_float_value: &mut Option<f64>, out: &egui::FullOutput) {
    for e in &out.platform_output.events {
        match e {
            egui::output::OutputEvent::Clicked(widget_info) => {
                if *ime_stage == ImeStage::Nothing {
                    *current_text_value = widget_info.current_text_value.clone();
                    *current_float_value = widget_info.value.clone();
                }
            }
            _ => (),
        }
    }
    *ime = out.platform_output.ime;
}

pub(crate) fn ime_part_a(gfx: &ctru::prelude::Gfx, apt: &ctru::prelude::Apt, ime_output: Option<egui::output::IMEOutput>, ime_stage: &mut ImeStage, current_text_value: &mut Option<String>, current_float_value: &mut Option<f64>, events: &mut Vec<egui::Event>) {
    if let Some(_) = ime_output {
        if *ime_stage == ImeStage::Nothing {
            use ctru::applets::swkbd;
            let mut kbd = swkbd::SoftwareKeyboard::new(
                swkbd::Kind::Normal,
                swkbd::ButtonConfig::LeftRight,
            );
            kbd.set_initial_text(
                current_text_value
                    .take()
                    .map(|x| std::borrow::Cow::Owned(x))
                    .or(current_float_value
                        .take()
                        .map(|x| std::borrow::Cow::Owned(x.to_string()))),
            );
            let (text, button) = kbd.launch(apt, gfx).unwrap();
            if button == swkbd::Button::Right {
                *current_text_value = Some(text);
                *ime_stage = ImeStage::START;
            } else {
                *ime_stage = ImeStage::CANCEL;
            }
        }
    }
    if ime_stage.add_event(events) {
        events.push(egui::Event::Text(
            current_text_value.take().unwrap_or_default(),
        ));
    }
    *ime_stage = ime_stage.next();
}
