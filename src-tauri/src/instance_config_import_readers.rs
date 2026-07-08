mod engine_ini;
mod game_ini;
mod game_user_settings;

pub(crate) use engine_ini::read_engine_ini;
pub(crate) use game_ini::read_game_ini;
pub(crate) use game_user_settings::read_game_user_settings;

const SERVER_SETTINGS: &[&str] = &["ServerSettings"];
const SESSION_SETTINGS: &[&str] = &["SessionSettings", "ServerSettings"];
const GAME_SESSION_SETTINGS: &[&str] = &["/Script/Engine.GameSession", "ServerSettings"];
const GAME_MODE_SETTINGS: &[&str] = &["/Script/ShooterGame.ShooterGameMode", ""];
const ENGINE_IP_NET_DRIVER_SETTINGS: &[&str] = &["/Script/OnlineSubsystemUtils.IpNetDriver"];
