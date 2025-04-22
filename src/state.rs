use core::fmt;
use std::collections::HashSet;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Api, StdError, StdResult, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, Key, Map, MultiIndex, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::error::ContractError;

pub const DID_PREFIX: &str = "did:c4e:"; // TODO make configurable on contract instatiating
                                         // const ADDRESS_DID_PREFIX: &str = constcat!(DID_PREFIX, "address:");

#[cw_serde]
pub struct DidDocument {
    pub id: Did,
    #[serde(
        serialize_with = "serialize_controllers",
        deserialize_with = "deserialize_controllers"
    )]
    pub controller: Vec<Controller>,
    // pub controller: Controllers,
    pub service: Vec<Service>,
}

pub(crate) trait Controllers {
    fn ensure_exist(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
    ) -> Result<(), ContractError>;
}

impl Controllers for Vec<Controller> {
    fn ensure_exist(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
    ) -> Result<(), ContractError> {
        for c in self {
            c.ensure_exist(store, did_docs)?;
        }
        Ok(())
    }
}

impl DidDocument {
    pub(crate) fn has_service(&self, service_did: &Did) -> bool {
        self.service
            .iter()
            .any(|service| &service.id == service_did)
    }

    // pub fn has_controller(&self, controller: &Controller) -> bool {
    //     self.controller.controllers().contains(controller)
    // }

    // pub fn has_any_controller(&self) -> bool {
    //     !self.controller.controllers().is_empty()
    // }

    pub(crate) fn has_controller(&self, controller: &Controller) -> bool {
        self.controller.contains(controller)
    }

    pub(crate) fn has_any_controller(&self) -> bool {
        !self.controller.is_empty()
    }

    pub(crate) fn ensure_controller(&self) -> Result<(), ContractError> {
        if !self.has_any_controller() {
            return Err(ContractError::DidDocumentNoController());
        }
        Ok(())
    }

    pub(crate) fn ensure_not_self_controlled(&self) -> Result<(), ContractError> {
        for c in &self.controller {
            if self.id.to_string() == c.to_string() {
                return Err(ContractError::SelfControlledDidDocumentNotAllowed());
            }
        }
        Ok(())
    }

    pub(crate) fn ensure_controllers_not_duplicated(&self) -> Result<(), ContractError> {
        let mut seen = HashSet::new();
        for controller in &self.controller {
            if !seen.insert(controller.to_string()) {
                return Err(ContractError::DuplicatedController(controller.to_string()));
            }
        }
        Ok(())
    }

    pub(crate) fn ensure_controllers_exist(
        &self,
        store: &mut dyn Storage,
        did_docs: &Map<String, DidDocument>,
    ) -> Result<(), ContractError> {
        self.controller.ensure_exist(store, did_docs)
    }

    pub(crate) fn ensure_signability(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
    ) -> Result<(), ContractError> {
        let mut already_checked: HashSet<String> = HashSet::new();
        self.can_be_signed(store, did_docs, &mut already_checked)
    }

    fn can_be_signed(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
        already_checked: &mut HashSet<String>,
    ) -> Result<(), ContractError> {
        for c in &self.controller {
            // for c in self.controller.controllers() {
            if c.is_signable() {
                return Ok(());
            } else {
                if already_checked.insert(c.to_string()) {
                    let did_doc_result: Result<DidDocument, StdError> =
                        did_docs.load(store, c.to_string());
                    match did_doc_result {
                        Ok(did_document) => {
                            if let Ok(_) =
                                did_document.can_be_signed(store, did_docs, already_checked)
                            {
                                return Ok(());
                            }
                        }
                        Err(e) => match e {
                            StdError::NotFound { .. } => (),
                            _ => {
                                return Err(ContractError::DidDocumentError(e));
                            }
                        },
                    }
                }
            }
        }
        Err(ContractError::DidDocumentUnsignable())
    }

    pub(crate) fn authorize(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
        sender: &Controller,
    ) -> Result<(), ContractError> {
        if !self.is_controlled_by(store, did_docs, sender)? {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    pub(crate) fn is_controlled_by(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
        controller: &Controller,
    ) -> Result<bool, ContractError> {
        let mut already_checked: HashSet<String> = HashSet::new();
        is_controller_of_internal(
            store,
            did_docs,
            &self.controller,
            controller,
            &mut already_checked,
        )
        // self.is_controller_internal(store, did_docs, controller, &mut already_checked)
    }

    fn is_controller_internal(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
        controller: &Controller,
        already_checked: &mut HashSet<String>,
    ) -> Result<bool, ContractError> {
        is_controller_of_internal(
            store,
            did_docs,
            &self.controller,
            controller,
            already_checked,
        )
    }

    pub fn is_valid(&self, api: &dyn Api) -> bool {
        self.id.is_valid() &&
            // !self.controller.controllers().iter().any(|c| !c.is_valid(api)) &&
            !self.controller.iter().any(|c| !c.is_valid(api)) &&
            !self.service.iter().any(|c| !c.is_valid())
    }

    pub fn ensure_valid(&self, api: &dyn Api) -> Result<(), ContractError> {
        self.id.ensure_valid()?;
        for c in &self.controller {
            c.ensure_valid(api)?
        }
        for s in &self.service {
            s.ensure_valid()?
        }
        Ok(())
    }
}

// pub fn ensure_controller_exist(store: &mut dyn Storage, did_docs: &Map<String, DidDocument>, controller: &Controller) -> Result<(), ContractError> {
//     if controller.is_did() {
//         if !did_docs.has(store, controller.to_string()) {
//             return Err(ContractError::DidControllerNotFound());
//         }
//     }
//     Ok(())
// }

pub fn is_controller_of(
    store: &dyn Storage,
    did_docs: &Map<String, DidDocument>,
    controllers: &Vec<Controller>,
    controller: &Controller,
) -> Result<bool, ContractError> {
    let mut already_checked: HashSet<String> = HashSet::new();
    is_controller_of_internal(
        store,
        did_docs,
        controllers,
        controller,
        &mut already_checked,
    )
}

fn is_controller_of_internal(
    store: &dyn Storage,
    did_docs: &Map<String, DidDocument>,
    controllers: &Vec<Controller>,
    controller: &Controller,
    already_checked: &mut HashSet<String>,
) -> Result<bool, ContractError> {
    for c in controllers {
        // for c in self.controller.controllers() {
        if c == controller {
            return Ok(true);
        }
        if c.is_did() {
            if already_checked.insert(c.to_string()) {
                let did_doc_result: Result<DidDocument, StdError> =
                    did_docs.load(store, c.to_string());
                match did_doc_result {
                    Ok(did_document) => {
                        let is_controller = did_document.is_controller_internal(
                            store,
                            did_docs,
                            controller,
                            already_checked,
                        )?;
                        if is_controller {
                            return Ok(true);
                        }
                    }
                    Err(e) => match e {
                        StdError::NotFound { .. } => (),
                        _ => {
                            return Err(ContractError::DidDocumentError(e));
                        }
                    },
                }
            }
        }
    }
    Ok(false)
}

// Custom serialization for controller field
fn serialize_controllers<S>(controller: &Vec<Controller>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if controller.len() == 1 {
        // If there's only one element, serialize it as a single Controller (not as an array)
        serializer.serialize_some(&controller[0].to_string())
    } else {
        // Otherwise, serialize as an array of Controllers
        let mut seq = serializer.serialize_seq(Some(controller.len()))?;
        for item in controller {
            seq.serialize_element(&item.to_string())?;
        }
        seq.end()
    }
}

// Custom deserialization for controller field
fn deserialize_controllers<'de, D>(deserializer: D) -> Result<Vec<Controller>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ControllerVisitor;
    impl<'de> Visitor<'de> for ControllerVisitor {
        type Value = Vec<Controller>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a sequence of controllers")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // If it's a string, wrap it in a Vec
            Ok(vec![Controller(value.to_string())])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut controllers = Vec::new();
            while let Some(controller) = seq.next_element()? {
                controllers.push(controller);
            }
            Ok(controllers)
        }
    }

    // Deserialize the input either as a single element or a sequence
    deserializer.deserialize_any(ControllerVisitor)
}
pub struct DidDocumentIndexes<'a> {
    pub controller: MultiIndex<'a, String, DidDocument, String>,
}

impl<'a> IndexList<DidDocument> for DidDocumentIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<DidDocument>> + '_> {
        let v: Vec<&dyn Index<DidDocument>> = vec![&self.controller];
        Box::new(v.into_iter())
    }
}

pub fn did_documents<'a>() -> IndexedMap<&'a str, DidDocument, DidDocumentIndexes<'a>> {
    let indexes = DidDocumentIndexes {
        controller: MultiIndex::new(
            // |_pk, d: &DidDocument| d.controller.controllers()[0].to_string(),
            |_pk, d: &DidDocument| d.controller[0].to_string(),
            "dids",
            "did_controller",
        ),
    };
    IndexedMap::new("escrows", indexes)
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

impl Service {
    pub fn is_valid(&self) -> bool {
        self.id.is_valid()
    }

    pub fn ensure_valid(&self) -> Result<(), ContractError> {
        self.id.ensure_valid()
    }
}

// #[derive(PartialEq, Debug, Clone, JsonSchema)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
pub struct Did(String);

impl cw_storage_plus::KeyDeserialize for Did {
    type Output = Did;

    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(Did::new(&String::from_vec(value)?))
    }
}

impl cw_storage_plus::KeyDeserialize for &Did {
    type Output = Did;

    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Self::Output::from_vec(value)
    }
}

impl<'a> PrimaryKey<'a> for Did {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_bytes())]
    }
}

impl<'a> Prefixer<'a> for Did {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_bytes())]
    }
}

impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Did, D::Error>
    where
        D: Deserializer<'de>,
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

impl From<String> for Did {
    // TODO maybe change to TryFrom<String>
    fn from(s: String) -> Self {
        Did::new(&s)
    }
}

impl From<&String> for Did {
    // TODO maybe change to TryFrom<String>
    fn from(s: &String) -> Self {
        Did::new(s)
    }
}

impl From<&str> for Did {
    // TODO maybe change to TryFrom<String>
    fn from(s: &str) -> Self {
        Did::new(s)
    }
}

impl From<Did> for String {
    // TODO maybe change to TryFrom<String>
    fn from(s: Did) -> Self {
        s.to_string()
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

    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the UTF-8 encoded address string as a byte array.
    ///
    /// This is equivalent to `address.as_str().as_bytes()`.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Utility for explicit conversion to `String`.
    #[inline]
    pub fn into_string(self) -> String {
        self.0
    }

    // pub fn new_address(s: &str) -> Self {
    //     let id: String = ADDRESS_DID_PREFIX.to_string() + s;
    //     Did(id)
    // }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn is_valid(&self) -> bool {
        Did::is_did(&self.0)
    }

    pub fn ensure_valid(&self) -> Result<(), ContractError> {
        if !self.is_valid() {
            return Err(ContractError::DidFormatError());
        }
        Ok(())
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
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Controller {
    fn deserialize<D>(deserializer: D) -> Result<Controller, D::Error>
    where
        D: Deserializer<'de>,
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

impl From<String> for Controller {
    // TODO maybe change to TryFrom<String>
    fn from(s: String) -> Self {
        Controller::new(&s)
    }
}

impl From<&String> for Controller {
    // TODO maybe change to TryFrom<String>
    fn from(s: &String) -> Self {
        Controller::new(s)
    }
}

impl From<&str> for Controller {
    // TODO maybe change to TryFrom<String>
    fn from(s: &str) -> Self {
        Controller::new(s)
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

    pub fn is_valid(&self, api: &dyn Api) -> bool {
        Controller::is_controller(api, &self.0)
    }

    pub fn ensure_valid(&self, api: &dyn Api) -> Result<(), ContractError> {
        if !self.is_valid(api) {
            return Err(ContractError::ControllerFormatError());
        }
        Ok(())
    }

    pub fn is_did(&self) -> bool {
        Did::is_did(&self.0)
    }

    pub fn is_signable(&self) -> bool {
        !self.is_did()
    }

    pub fn is_controller(api: &dyn Api, s: &str) -> bool {
        if let Err(_) = api.addr_validate(s) {
            return Did::is_did(s);
        }
        true
    }

    pub(crate) fn ensure_exist(
        &self,
        store: &dyn Storage,
        did_docs: &Map<String, DidDocument>,
    ) -> Result<(), ContractError> {
        if self.is_did() {
            if !did_docs.has(store, self.to_string()) {
                return Err(ContractError::DidControllerNotFound());
            }
        }
        Ok(())
    }

    
    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()   
    }
}

pub(crate) trait ToEventData {
    fn to_event_data(&self) -> String;
}

impl ToEventData for Vec<Controller> {
    fn to_event_data(&self) -> String {
        self.iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl ToEventData for Vec<Service> {
    fn to_event_data(&self) -> String {
        self.iter()
            .map(|s| s.id.to_string())
            .collect::<Vec<String>>()
            .join(",")
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
        assert_eq!(serialized_value, expected_json);

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
    fn test_did_serialization_one_controller() {
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
            // controller: Controllers(vec![Controller::new("controller1")]),
            controller: vec![Controller::new("controller1")],
            service: vec![service1, service2],
        };

        // Serialize to JSON
        let serialized = to_string(&did).unwrap();
        println!("FFFFF {}", serialized);
        let expected_json = json!({
            "id": "did1",
            "controller": "controller1",
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
        // assert_eq!(deserialized.controller, Controllers(vec![Controller::new("controller1")]));
        assert_eq!(
            deserialized.controller,
            vec![Controller::new("controller1")]
        );
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

    #[test]
    fn test_did_serialization_many_controllers() {
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
            // controller: Controllers(vec![Controller::new("controller1"), Controller::new("controller2")]),
            controller: vec![
                Controller::new("controller1"),
                Controller::new("controller2"),
            ],
            service: vec![service1, service2],
        };

        // Serialize to JSON
        let serialized = to_string(&did).unwrap();
        println!("FFFFF {}", serialized);
        let expected_json = json!({
            "id": "did1",
            "controller": ["controller1","controller2"],
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
        assert_eq!(deserialized, did);
        assert_eq!(deserialized.id, Did::new("did1"));
        // assert_eq!(deserialized.controller, Controllers(vec![Controller::new("controller1"), Controller::new("controller2")]));
        assert_eq!(
            deserialized.controller,
            vec![
                Controller::new("controller1"),
                Controller::new("controller2")
            ]
        );
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
