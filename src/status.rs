use std::fmt::Display;

#[derive(Debug)]
pub enum BotStatus {
    WaitingCommand,
    RecordTitle,
    RecordContent,
    RecordRemark,
    HandleOtherCommand,
}

impl From<String> for BotStatus {
    fn from(value: String) -> Self {
        match value.as_str() { 
            "WaitingCommand" => BotStatus::WaitingCommand,
            "RecordTitle" => BotStatus::RecordTitle,
            "RecordContent" => BotStatus::RecordContent,
            "RecordRemark" => BotStatus::RecordRemark,
            "HandleOtherCommand" => BotStatus::HandleOtherCommand,
            _ => BotStatus::WaitingCommand,
        }
    }
}

impl Display for BotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BotStatus::WaitingCommand => String::from("WaitingCommand"),
            BotStatus::RecordTitle => String::from("RecordTitle"),
            BotStatus::RecordContent => String::from("RecordContent"),
            BotStatus::RecordRemark => String::from("RecordRemark"),
            BotStatus::HandleOtherCommand => String::from("HandleOtherCommand"),
        };
        write!(f, "{}", str)
    }
}
