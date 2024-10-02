use core::fmt;
use std::collections::HashSet;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Api, StdError, Storage};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use constcat::concat as constcat;

use crate::error::ContractError;

const DID_PREFIX: &str = "didc4e:c4e:"; // TODO make configurable on contract instatiating
const ADDRESS_DID_PREFIX: &str = constcat!(DID_PREFIX, "address:");

#[cw_serde]
pub struct DidDocument {
    pub id: Did,
    pub controller: Vec<Controller>,
    pub service: Vec<Service>,
}

impl DidDocument {
    pub fn has_service(&self, service_did: &Did) -> bool {
        self.service.iter().any(|service| &service.id == service_did)
    }

    pub fn has_controller(&self, controller: &Controller) -> bool {
        self.controller.contains(controller)
    }

    // TODO make not public for axternal crate
    pub fn is_controlled_by(&self, store: &dyn Storage, did_docs: &Map<String, DidDocument>, controller: &Controller) -> Result<bool, ContractError> {
        let mut already_checked: HashSet<String> = HashSet::new();
        self.is_controller_internal(store, did_docs, controller, &mut already_checked)
    }

    fn is_controller_internal(&self, store: &dyn Storage, did_docs: &Map<String, DidDocument>, controller: &Controller, already_checked: &mut HashSet<String>)  -> Result<bool, ContractError> {
        for c in &self.controller {
            // println!("did: {}, controller: {}, external_controller: {}", self.id.to_string(), c, &controller);
            if c == controller {
                // println!("controller: {} == external_controller: {}", c, controller);
                return Ok(true);
            }
            if c.is_did() {
                // println!("controller: {} is did", c);
                if already_checked.insert(c.to_string()) {
                    let did_doc_result: Result<DidDocument, StdError> = did_docs.load(store, c.to_string());
                    match did_doc_result {
                        Ok(did_document) => {
                            let is_controller = did_document.is_controller_internal(store, did_docs, controller, already_checked)?;
                            if is_controller {
                                return Ok(true);
                            }
                        },
                        Err(e) => match e {
                            StdError::NotFound{ .. } => (),
                            _ => {
                                return Err(ContractError::DidDocumentError(e));
                            },
                        },
                    }
                }
            }
        }
        Ok(false)
    }
}

#[cw_serde]
pub struct Authentication {
    pub id: Did,
    #[serde(rename = "type")]
    pub a_type: String,
    pub controller: Did,
    // pub c4e_address: String,
}

#[cw_serde]
pub struct Service {
    pub id: Did,
    #[serde(rename = "type")]
    pub a_type: String,
    pub service_endpoint: String,
}
 
#[derive(PartialEq, Debug, Clone, JsonSchema)]
pub struct Did(String);

impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Did, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(Did(s))
    }
}

// impl ToString for Did {
//     #[inline]
//     fn to_string(&self) -> String {
//         self.0.clone()
//     }
// }

impl From<String> for Did { // TODO maybe change to TryFrom<String>
    fn from(s: String) -> Self {
        Did::new(&s)
    }
}

// Implement Display to allow conversion from MyStruct to String (automatically implements ToString)
impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
        // write!(f, "{}", self.content)
    }
}

impl PartialEq<String> for &Did {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Did> for String {
    fn eq(&self, other: &Did) -> bool {
        *self == other.0
    }
}

// impl PartialEq for Did {
//     #[inline]
//     fn eq(&self, other: &Did) -> bool {
//         self.0 == other.0
//     }
// }

impl Did {
    pub fn new(s: &str) -> Self {
        Did(s.to_string())
    }

    pub fn new_address(s: &str) -> Self {
        let id: String = ADDRESS_DID_PREFIX.to_string() + s;
        Did(id)
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn validate(&self) -> bool {
        Did::is_did(&self.0)
    }

    pub fn is_did(s: &str) -> bool {
        s.starts_with(DID_PREFIX)
    }
}


#[derive(PartialEq, Debug, Clone, JsonSchema)]
pub struct Controller(String);

impl Serialize for Controller {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Controller {
    fn deserialize<D>(deserializer: D) -> Result<Controller, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(Controller(s))
    }
}

// impl ToString for Controller {
//     #[inline]
//     fn to_string(&self) -> String {
//         self.0.clone()
//     }
// }

impl PartialEq<String> for &Controller {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Controller> for String {
    fn eq(&self, other: &Controller) -> bool {
        *self == other.0
    }
}

impl From<String> for Controller {  // TODO maybe change to TryFrom<String>
    fn from(s: String) -> Self {
        Controller::new(&s)
    }
}

// Implement Display to allow conversion from MyStruct to String (automatically implements ToString)
impl fmt::Display for Controller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
        // write!(f, "{}", self.content)
    }
}

impl Controller {
    pub fn new(s: &str) -> Self {
        Controller(s.to_string())
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn validate(&self, api: &dyn Api) -> bool {
        Controller::is_controller(api, &self.0)
    }

    pub fn is_did(&self) -> bool {
        Did::is_did(&self.0)
    }

    pub fn is_controller(api: &dyn Api, s: &str) -> bool {
        if let Err(_) = api.addr_validate(s) {
            return Did::is_did(s)
        }
        true
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, json, to_string};

    #[test]
    fn test_service_serialization() {
        // Create a sample Service struct
        let service = Service {
            id: Did::new("service1"),
            a_type: "SomeServiceType".to_string(),
            service_endpoint: "https://example.com".to_string(),
        };

        // Serialize to JSON
        let serialized = to_string(&service).unwrap();
        let expected_json = json!({
            "id": "service1",
            "type": "SomeServiceType",
            "service_endpoint": "https://example.com"
        });

        let serialized_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Compare with expected JSON string
        assert_eq!(serialized_value , expected_json);

        // Deserialize back to struct
        let deserialized: Service = from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, Did::new("service1"));
        assert_eq!(deserialized.a_type, "SomeServiceType");
        assert_eq!(deserialized.service_endpoint, "https://example.com");
    }

    #[test]
    fn test_authentocation_serialization() {
        // Create a sample Authentocation struct
        let authentocation = Authentication {
            id: Did::new("auth1"),
            a_type: "SomeAuthType".to_string(),
            controller: Did::new("controller1"),
            // c4e_address: "address1".to_string(),
        };

        // Serialize to JSON
        let serialized = serde_json::to_string(&authentocation).unwrap();
        let expected_json = json!({
            "id": "auth1",
            "type": "SomeAuthType",
            "controller": "controller1",
            // "c4e_address": "address1"
        });

        // Deserialize both to serde_json::Value to compare unordered JSON
        let serialized_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        // let expected_value: serde_json::Value = expected_json;

        // Compare the values, which ignores order of keys
        assert_eq!(serialized_value, expected_json);

        // Deserialize back to struct
        let deserialized: Authentication = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, Did::new("auth1"));
        assert_eq!(deserialized.a_type, "SomeAuthType");
        assert_eq!(deserialized.controller, Did::new("controller1"));
        // assert_eq!(deserialized.c4e_address, "address1");
    }

    #[test]
    fn test_did_serialization() {
        // Create a sample Did struct with multiple services
        let service1 = Service {
            id: Did::new("service1"),
            a_type: "ServiceType1".to_string(),
            service_endpoint: "https://service1.com".to_string(),
        };
        let service2 = Service {
            id: Did::new("service2"),
            a_type: "ServiceType2".to_string(),
            service_endpoint: "https://service2.com".to_string(),
        };
        let did = DidDocument {
            id: Did::new("did1"),
            controller: vec![Controller::new("controller1")],
            service: vec![service1, service2],
        };

        // Serialize to JSON
        let serialized = to_string(&did).unwrap();
        let expected_json = json!({
            "id": "did1",
            "controller": ["controller1"],
            "service": [
                {
                    "id": "service1",
                    "type": "ServiceType1",
                    "service_endpoint": "https://service1.com"
                },
                {
                    "id": "service2",
                    "type": "ServiceType2",
                    "service_endpoint": "https://service2.com"
                }
            ]
        });

        let serialized_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Compare with expected JSON string
        assert_eq!(serialized_value, expected_json);

        // Deserialize back to struct
        let deserialized: DidDocument = from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, Did::new("did1"));
        assert_eq!(deserialized.controller, vec![Controller::new("controller1")]);
        assert_eq!(deserialized.service.len(), 2);
        assert_eq!(deserialized.service[0].id, Did::new("service1"));
        assert_eq!(deserialized.service[0].a_type, "ServiceType1");
        assert_eq!(
            deserialized.service[0].service_endpoint,
            "https://service1.com"
        );
        assert_eq!(deserialized.service[1].id, Did::new("service2"));
        assert_eq!(deserialized.service[1].a_type, "ServiceType2");
        assert_eq!(
            deserialized.service[1].service_endpoint,
            "https://service2.com"
        );
    }
}
// {
//   "@context": "https://www.w3.org/ns/did/v1",
//   "id": "c4e:did:chargera:cp:dafsdsfdssdfss",
//   "authentication": [
//     {
//       "id": "c4e:did:chargera:cp:dafsdsfdssdfss#key-1",
//       "type": "Ed25519VerificationKey2018",
//       "controller": "c4e:did:chargera:us:dsfdfdsafaaffsfdsafa",
//       "publicKeyBase58": "GfHFG7H8jfkd83Kfjkf8K3j3jFJKJDdjd3"
//     }
//   ],
//   "service": [
//     {
//       "id": "c4e:did:chargera:chargera#chargera-application",
//       "type": "Chargera",
//       "serviceEndpoint": "https://chargera.io"
//     },
//     {
//       "id": "c4e:did:chargera:chargera#chargera-cp",
//       "type": "ChargerpChargingPoint",
//       "serviceEndpoint": "https://chargera.io/cp/3/evse/234"
//     }
//   ]
// }
