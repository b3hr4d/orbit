use candid::{CandidType, Deserialize};
use ic_stable_structures::{storable::Bound, Storable};
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum AccessRole {
    Admin = 0,
    User = 1,
    Guest = 2,
}

impl From<AccessRole> for u8 {
    fn from(role: AccessRole) -> Self {
        role as u8
    }
}

impl TryFrom<u8> for AccessRole {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AccessRole::Admin),
            1 => Ok(AccessRole::User),
            2 => Ok(AccessRole::Guest),
            _ => Err(()),
        }
    }
}

impl FromStr for AccessRole {
    type Err = ();

    fn from_str(variant: &str) -> Result<AccessRole, Self::Err> {
        match variant {
            "admin" => Ok(AccessRole::Admin),
            "user" => Ok(AccessRole::User),
            "guest" => Ok(AccessRole::Guest),
            _ => Err(()),
        }
    }
}

impl Display for AccessRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessRole::Admin => write!(f, "admin"),
            AccessRole::User => write!(f, "user"),
            AccessRole::Guest => write!(f, "guest"),
        }
    }
}

impl Storable for AccessRole {
    fn to_bytes(&self) -> Cow<[u8]> {
        let access_role_unit: u8 = self.to_owned().into();
        Cow::Owned(access_role_unit.to_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let access_role_unit = u8::from_bytes(bytes);
        AccessRole::try_from(access_role_unit).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_role_string_conversion() {
        assert_eq!(AccessRole::Admin.to_string(), "admin");
        assert_eq!(AccessRole::User.to_string(), "user");
        assert_eq!(AccessRole::Guest.to_string(), "guest");
    }

    #[test]
    fn test_access_role_from_str() {
        assert_eq!(AccessRole::from_str("admin").unwrap(), AccessRole::Admin);
        assert_eq!(AccessRole::from_str("user").unwrap(), AccessRole::User);
        assert_eq!(AccessRole::from_str("guest").unwrap(), AccessRole::Guest);
    }

    #[test]
    fn test_access_role_from_number() {
        assert_eq!(AccessRole::try_from(0).unwrap(), AccessRole::Admin);
        assert_eq!(AccessRole::try_from(1).unwrap(), AccessRole::User);
        assert_eq!(AccessRole::try_from(2).unwrap(), AccessRole::Guest);
    }
}