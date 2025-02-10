use std::fmt::Display;

#[derive(PartialEq, Clone, Copy)]
pub enum DroneVendor {
    RustafarianDrone,
    LockheedRustin,
    RustyDrone,
    RustBustersDrone,
    CppEnjoyersDrone,
    RustezeDrone,
    GetDroned,
    RustRoveri,
    MyDrone,
    Unknown,
}

impl Display for DroneVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DroneVendor::RustafarianDrone => "RustafarianDrone".to_string(),
            DroneVendor::LockheedRustin => "LockheedRustin".to_string(),
            DroneVendor::RustyDrone => "RustyDrone".to_string(),
            DroneVendor::RustBustersDrone => "RustBustersDrone".to_string(),
            DroneVendor::CppEnjoyersDrone => "CppEnjoyersDrone".to_string(),
            DroneVendor::RustezeDrone => "RustezeDrone".to_string(),
            DroneVendor::GetDroned => "GetDroned".to_string(),
            DroneVendor::RustRoveri => "RustRoveri".to_string(),
            DroneVendor::MyDrone => "MyDrone".to_string(),
            DroneVendor::Unknown => "Unknown".to_string(),
        };
        write!(f, "{s}")
    }
}
