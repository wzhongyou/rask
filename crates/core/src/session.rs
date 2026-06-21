use crate::Message;

pub struct Session {
    pub model: String,
    pub messages: Vec<Message>,
}

impl Session {
    pub fn new(model: impl Into<String>) -> Self {
        let now = chrono::Local::now();
        let system = format!(
            "Today is {}. You are a helpful assistant.",
            now.format("%Y-%m-%d %A")
        );
        Self {
            model: model.into(),
            messages: vec![Message { role: "system".into(), content: system }],
        }
    }

    pub fn push_user(&mut self, content: impl Into<String>) {
        self.messages.push(Message::user(content));
    }

    pub fn push_assistant(&mut self, content: impl Into<String>) {
        self.messages.push(Message::assistant(content));
    }
}
