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

#[derive(PartialEq, Clone, Copy)]
pub enum ClientVendor {
    LeonardosClient,
    LucasClient,
    Unknown,
}

impl Display for ClientVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ClientVendor::LeonardosClient => "LeonardosClient",
            ClientVendor::LucasClient => "LucasClient",
            ClientVendor::Unknown => "Unknown",
        }
        .to_string();
        write!(f, "{s}")
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ServerVendor {
    GinosServer,
    Unknown,
}

impl Display for ServerVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ServerVendor::GinosServer => "GinosServer",
            ServerVendor::Unknown => "Unknown",
        }
        .to_string();
        write!(f, "{s}")
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Vendor {
    Drone(DroneVendor),
    Client(ClientVendor),
    Server(ServerVendor),
}

impl Display for Vendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Vendor::Drone(d) => d.to_string(),
            Vendor::Client(c) => c.to_string(),
            Vendor::Server(s) => s.to_string(),
        };
        write!(f, "{s}")
    }
}
