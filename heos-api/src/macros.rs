// got docus from:https://fromherotozero.dev/blog/introduction-to-rust-macros/

macro_rules! cmd {

    ($name:ident :: $send_name:literal  -> $returns:ty ) => {
        pub struct $name;
        impl HeosCommand for $name {
            type CommandResponseType = $returns;
            fn payload(&self) -> String {
                $send_name.to_string()
            }
        }
    };

    ($name:ident :: $send_name:literal ($($param_name:ident : $param_type:ty ), + ) -> $returns:ty ) => {
        pub struct $name {
            $(
                pub  $param_name : $param_type,
            )*
        }
        impl HeosCommand for $name {
            type CommandResponseType = $returns;
            fn payload(&self) -> String {
                let mut temp_vec = Vec::new();
                $(
                    temp_vec.push(format!("{}={}", stringify!($param_name), self.$param_name));
                )*
                format!("{}?{}", $send_name.to_string(), temp_vec.join("&"))
            }
        }
    };
}

macro_rules! jason_parser {
    ($t:ty) => {
        impl TryFrom<CommandResponse> for $t {
            type Error = HeosError;

            fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
                serde_json::from_value(value.payload)
                    .context(format!(
                        "Failed to parse {} from command response.",
                        stringify!($t)
                    ))
                    .map_err(|e| e.into())
            }
        }
    };
}
macro_rules! json_option_parser {
    ($t:ty) => {
        impl TryFrom<CommandResponse> for Option<$t> {
            type Error = HeosError;

            fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
                match value.payload {
                    Value::Object(map) if !map.is_empty() => {
                        (serde_json::from_value(Value::Object(map))
                            .context(format!(
                                "Failed to parse {} from command response.",
                                stringify!($t)
                            ))
                            .map(|res| Some(res)))
                            .map_err(|e| e.into())

                    }
                    _ => Ok(None)
                }
            }
        }
    };
}

macro_rules! qs_parser {
    ($t:ty) => {
        impl TryFrom<CommandResponse> for $t {
            type Error = HeosError;

            fn try_from(value: CommandResponse) -> Result<Self, Self::Error> {
                qs::from_str(&value.message)
                    .context(format!(
                        "Failed to parse CommandResponse for {}.",
                        stringify!($t)
                    ))
                    .map_err(|e| e.into())
            }
        }
    };
}
