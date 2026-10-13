use std::path::{Path, PathBuf};

use anyhow::Context;
use tokio::fs;

use super::{RuntimeEvent, TranscriptMessage};

#[derive(Clone, Debug)]
pub struct TranscriptStore {
    path: PathBuf,
    events: Vec<RuntimeEvent>,
}

impl TranscriptStore {
    pub async fn load(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let text = match fs::read_to_string(&path).await {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
        };
        let mut events = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            events.push(serde_json::from_str(line).context("parse transcript JSONL line")?);
        }
        Ok(Self { path, events })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn events(&self) -> &[RuntimeEvent] {
        &self.events
    }

    pub async fn append(&mut self, event: &RuntimeEvent) -> anyhow::Result<()> {
        self.events.push(event.clone());
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut line = serde_json::to_string(event)?;
        line.push('\n');
        use tokio::io::AsyncWriteExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        file.write_all(line.as_bytes()).await?;
        Ok(())
    }

    pub fn current_transcript(&self) -> Vec<TranscriptMessage> {
        let mut messages = Vec::new();
        let mut assistant_turn: Option<(String, String)> = None;
        let flush = |messages: &mut Vec<TranscriptMessage>,
                     assistant_turn: &mut Option<(String, String)>| {
            if let Some((_turn_id, content)) = assistant_turn.take() {
                messages.push(TranscriptMessage {
                    role: "assistant".to_string(),
                    content,
                });
            }
        };
        for event in &self.events {
            match event {
                RuntimeEvent::TurnStarted { user_text, .. } => {
                    flush(&mut messages, &mut assistant_turn);
                    messages.push(TranscriptMessage {
                        role: "user".to_string(),
                        content: user_text.clone(),
                    });
                }
                RuntimeEvent::AssistantDelta { turn_id, delta, .. } => match &mut assistant_turn {
                    Some((active, content)) if active == turn_id => content.push_str(delta),
                    _ => {
                        flush(&mut messages, &mut assistant_turn);
                        assistant_turn = Some((turn_id.clone(), delta.clone()));
                    }
                },
                RuntimeEvent::TurnFinished { .. } | RuntimeEvent::TurnFailed { .. } => {
                    flush(&mut messages, &mut assistant_turn);
                }
                _ => {}
            }
        }
        flush(&mut messages, &mut assistant_turn);
        messages
    }
}
