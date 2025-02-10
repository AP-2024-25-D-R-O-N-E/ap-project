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
    LeDronJamesDrone,
    SkyLinkDrone,
    MyDrone,
    Unknown,
}

impl Display for DroneVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DroneVendor::RustafarianDrone => "RustafarianDrone",
            DroneVendor::LockheedRustin => "LockheedRustin",
            DroneVendor::RustyDrone => "RustyDrone",
            DroneVendor::RustBustersDrone => "RustBustersDrone",
            DroneVendor::CppEnjoyersDrone => "CppEnjoyersDrone",
            DroneVendor::RustezeDrone => "RustezeDrone",
            DroneVendor::GetDroned => "GetDroned",
            DroneVendor::RustRoveri => "RustRoveri",
            DroneVendor::MyDrone => "MyDrone",
            DroneVendor::LeDronJamesDrone => "LeDronJamesDrone",
            DroneVendor::Unknown => "Unknown",
            DroneVendor::SkyLinkDrone => "SkyLinkDrone",
        }
        .to_string();
        write!(f, "{s}")
    }
}
