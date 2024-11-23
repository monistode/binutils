use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawRegisterGroup {
    pub length: u8,
    pub registers: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RawArgumentDefinition {
    #[serde(rename = "register")]
    Register { group: String },
    #[serde(rename = "data_address")]
    DataAddress { bits: u8 },
    #[serde(rename = "text_address")]
    TextAddress { bits: u8 },
    #[serde(rename = "padding")]
    Padding { bits: u8 },
    #[serde(rename = "immediate")]
    Immediate { bits: u8 },
}

#[derive(Debug, Deserialize)]
pub struct RawCommandDefinition {
    pub mnemonic: String,
    pub opcode: u8,
    #[serde(default)]
    pub arguments: Vec<RawArgumentDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct RawDefinition {
    pub opcode_length: u8,
    pub opcode_offset: u8,
    pub text_byte_length: u8,
    pub data_byte_length: u8,
    pub text_address_size: u8,
    pub data_address_size: u8,
    pub register_groups: HashMap<String, RawRegisterGroup>,
    pub commands: Vec<RawCommandDefinition>,
}

impl RawDefinition {
    pub fn from_str(s: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(s)
    }
}

#[derive(Debug, Clone)]
pub struct RegisterGroup {
    pub length: u8,
    pub registers: Vec<String>,
}

impl From<RawRegisterGroup> for RegisterGroup {
    fn from(raw: RawRegisterGroup) -> Self {
        RegisterGroup {
            length: raw.length,
            registers: raw.registers,
        }
    }
}

#[derive(Debug)]
pub enum ArgumentDefinition {
    Register { group: RegisterGroup },
    DataAddress { bits: u8 },
    TextAddress { bits: u8 },
    Padding { bits: u8 },
    Immediate { bits: u8 },
}

impl ArgumentDefinition {
    pub fn size(&self) -> u8 {
        match self {
            ArgumentDefinition::Register { group } => group.length,
            ArgumentDefinition::DataAddress { bits } => *bits,
            ArgumentDefinition::TextAddress { bits } => *bits,
            ArgumentDefinition::Padding { bits } => *bits,
            ArgumentDefinition::Immediate { bits } => *bits,
        }
    }
}

impl TryFrom<(RawArgumentDefinition, HashMap<String, RegisterGroup>)> for ArgumentDefinition {
    type Error = &'static str;

    fn try_from(
        raw: (RawArgumentDefinition, HashMap<String, RegisterGroup>),
    ) -> Result<Self, Self::Error> {
        match raw {
            (RawArgumentDefinition::Register { group }, groups) => match groups.get(&group) {
                Some(g) => Ok(ArgumentDefinition::Register { group: g.clone() }),
                None => Err("Register group not found"),
            },
            (RawArgumentDefinition::DataAddress { bits }, _) => {
                Ok(ArgumentDefinition::DataAddress { bits })
            }
            (RawArgumentDefinition::TextAddress { bits }, _) => {
                Ok(ArgumentDefinition::TextAddress { bits })
            }
            (RawArgumentDefinition::Padding { bits }, _) => {
                Ok(ArgumentDefinition::Padding { bits })
            }
            (RawArgumentDefinition::Immediate { bits }, _) => {
                Ok(ArgumentDefinition::Immediate { bits })
            }
        }
    }
}

#[derive(Debug)]
pub struct CommandDefinition {
    pub mnemonic: String,
    pub opcode: u8,
    pub arguments: Vec<ArgumentDefinition>,
}

impl TryFrom<(RawCommandDefinition, HashMap<String, RegisterGroup>)> for CommandDefinition {
    type Error = &'static str;

    fn try_from(
        raw: (RawCommandDefinition, HashMap<String, RegisterGroup>),
    ) -> Result<Self, Self::Error> {
        let (raw, groups) = raw;
        let arguments = raw
            .arguments
            .into_iter()
            .map(|a| ArgumentDefinition::try_from((a, groups.clone())))
            .collect::<Result<Vec<ArgumentDefinition>, Self::Error>>()?;

        Ok(CommandDefinition {
            mnemonic: raw.mnemonic,
            opcode: raw.opcode,
            arguments,
        })
    }
}

impl CommandDefinition {
    pub fn arguments_size(&self) -> u8 {
        self.arguments.iter().map(|a| a.size()).sum()
    }
}

#[derive(Debug)]
pub struct Definition {
    pub opcode_length: u8,
    pub opcode_offset: u8,
    pub text_byte_length: u8,
    pub data_byte_length: u8,
    pub address_size: u8,
    pub register_groups: HashMap<String, RegisterGroup>,
    pub commands: Vec<CommandDefinition>,
}

impl TryFrom<RawDefinition> for Definition {
    type Error = String;

    fn try_from(raw: RawDefinition) -> Result<Self, Self::Error> {
        let register_groups: HashMap<String, RegisterGroup> = raw
            .register_groups
            .into_iter()
            .map(|(k, v)| (k, RegisterGroup::from(v)))
            .collect();

        if raw.text_address_size != raw.data_address_size {
            return Err("Differing text and data address sizes are not supported".to_string());
        }

        let definition = Definition {
            opcode_length: raw.opcode_length,
            opcode_offset: raw.opcode_offset,
            text_byte_length: raw.text_byte_length,
            data_byte_length: raw.data_byte_length,
            address_size: raw.text_address_size,
            register_groups: register_groups.clone(),
            commands: raw
                .commands
                .into_iter()
                .map(|c| CommandDefinition::try_from((c, register_groups.clone())))
                .collect::<Result<Vec<CommandDefinition>, &'static str>>()?,
        };

        // Check whether all addresses are the same size
        for command in &definition.commands {
            for argument in &command.arguments {
                match argument {
                    ArgumentDefinition::DataAddress { bits } => {
                        if *bits != definition.address_size {
                            return Err("Data address size mismatch".to_string());
                        }
                    }
                    ArgumentDefinition::TextAddress { bits } => {
                        if *bits != definition.address_size {
                            return Err("Text address size mismatch".to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check whether all commands have unique opcodes
        let mut opcodes = HashMap::new();
        for command in &definition.commands {
            if opcodes.contains_key(&command.opcode) {
                return Err(format!(
                    "Duplicate opcode: {}, both for {} and {}",
                    command.opcode, opcodes[&command.opcode], command.mnemonic
                ));
            }
            opcodes.insert(command.opcode, &command.mnemonic);
        }

        // Check divisibility of command sizes by text byte length
        for command in &definition.commands {
            if (definition.opcode_length + command.arguments_size()) % definition.text_byte_length
                != 0
            {
                return Err(format!(
                    "Command size not divisible by text byte length: {} ({} bits)",
                    command.mnemonic,
                    command.arguments_size() + definition.opcode_length
                ));
            }
        }

        Ok(definition)
    }
}

impl TryFrom<String> for Definition {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let raw = RawDefinition::from_str(&s).map_err(|_| "Failed to parse YAML")?;
        Definition::try_from(raw)
    }
}
