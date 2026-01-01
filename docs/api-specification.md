# DID Contract API Specification

## Overview

This document provides a comprehensive API reference for the DID (Decentralized Identifier) Contract. The contract implements W3C DID specification for decentralized identity management on the Chain4Energy blockchain.

## Contract Interface

### Instantiation

The contract requires no parameters for instantiation.

#### Instantiate Message

```json
{}
```

#### Example
```bash
c4ed tx wasm instantiate <code_id> '{}' \
  --from admin \
  --label "did-contract" \
  --gas auto \
  --gas-adjustment 1.5
```

## Execute Messages

### 1. Create DID Document

Creates a new DID document with controllers and services.

#### Schema
```json
{
  "create_did_document": {
    "did_doc": {
      "id": "string",
      "controller": ["string"],
      "service": [
        {
          "id": "string",
          "a_type": "string",
          "service_endpoint": "string"
        }
      ]
    }
  }
}
```

#### Parameters
- `did_doc.id` (string): DID identifier (format: `did:c4e:<identifier>`)
- `did_doc.controller` (array): List of controller addresses or DIDs
- `did_doc.service` (array): List of service endpoints (optional)

#### Validation Rules
- DID must follow `did:c4e:` format
- DID must not already exist
- Controllers cannot contain duplicates
- Services cannot contain duplicates
- Controllers must exist (if DID controllers)
- Cannot be self-controlled

#### Example
```json
{
  "create_did_document": {
    "did_doc": {
      "id": "did:c4e:alice",
      "controller": ["c4e1abc...def"],
      "service": [
        {
          "id": "did:c4e:alice#messaging",
          "a_type": "MessagingService",
          "service_endpoint": "https://alice.example.com/messaging"
        }
      ]
    }
  }
}
```

#### Response Events
```json
{
  "type": "wasm-create_did_document",
  "attributes": [
    {"key": "did", "value": "did:c4e:alice"},
    {"key": "controllers", "value": "c4e1abc...def"},
    {"key": "services", "value": "did:c4e:alice#messaging"}
  ]
}
```

### 2. Update DID Document

Updates an existing DID document.

#### Schema
```json
{
  "update_did_document": {
    "new_did_doc": {
      "id": "string",
      "controller": ["string"],
      "service": [
        {
          "id": "string",
          "a_type": "string",
          "service_endpoint": "string"
        }
      ]
    }
  }
}
```

#### Authorization
- Caller must be a controller of the existing DID document

#### Example
```json
{
  "update_did_document": {
    "new_did_doc": {
      "id": "did:c4e:alice",
      "controller": ["c4e1abc...def", "c4e1xyz...123"],
      "service": [
        {
          "id": "did:c4e:alice#messaging",
          "a_type": "MessagingService", 
          "service_endpoint": "https://alice.newdomain.com/messaging"
        },
        {
          "id": "did:c4e:alice#auth",
          "a_type": "AuthenticationService",
          "service_endpoint": "https://alice.newdomain.com/auth"
        }
      ]
    }
  }
}
```

#### Response Events
```json
{
  "type": "wasm-update_did_document",
  "attributes": [
    {"key": "did", "value": "did:c4e:alice"},
    {"key": "old_controllers", "value": "c4e1abc...def"},
    {"key": "new_controllers", "value": "c4e1abc...def,c4e1xyz...123"},
    {"key": "old_services", "value": "did:c4e:alice#messaging"},
    {"key": "new_services", "value": "did:c4e:alice#messaging,did:c4e:alice#auth"}
  ]
}
```

### 3. Add Controller

Adds a new controller to an existing DID document.

#### Schema
```json
{
  "add_controller": {
    "did": "string",
    "controller": "string"
  }
}
```

#### Parameters
- `did` (string): Target DID identifier
- `controller` (string): New controller address or DID

#### Authorization
- Caller must be a controller of the target DID

#### Example
```json
{
  "add_controller": {
    "did": "did:c4e:alice",
    "controller": "c4e1new...controller"
  }
}
```

#### Response Events
```json
{
  "type": "wasm-add_controller",
  "attributes": [
    {"key": "did", "value": "did:c4e:alice"},
    {"key": "new_controller", "value": "c4e1new...controller"}
  ]
}
```

### 4. Delete Controller

Removes a controller from an existing DID document.

#### Schema
```json
{
  "delete_controller": {
    "did": "string",
    "controller": "string"
  }
}
```

#### Validation Rules
- Cannot remove the last controller
- Controller must exist in the document

#### Example
```json
{
  "delete_controller": {
    "did": "did:c4e:alice",
    "controller": "c4e1old...controller"
  }
}
```

#### Response Events
```json
{
  "type": "wasm-delete_controller",
  "attributes": [
    {"key": "did", "value": "did:c4e:alice"},
    {"key": "deleted_controller", "value": "c4e1old...controller"}
  ]
}
```

### 5. Add Service

Adds a new service to an existing DID document.

#### Schema
```json
{
  "add_service": {
    "did": "string",
    "service": {
      "id": "string",
      "a_type": "string", 
      "service_endpoint": "string"
    }
  }
}
```

#### Example
```json
{
  "add_service": {
    "did": "did:c4e:alice",
    "service": {
      "id": "did:c4e:alice#storage",
      "a_type": "StorageService",
      "service_endpoint": "https://alice.example.com/storage"
    }
  }
}
```

#### Response Events
```json
{
  "type": "wasm-add_service",
  "attributes": [
    {"key": "did", "value": "did:c4e:alice"},
    {"key": "new_service", "value": "did:c4e:alice#storage"}
  ]
}
```

### 6. Delete Service

Removes a service from an existing DID document.

#### Schema
```json
{
  "delete_service": {
    "did": "string",
    "service_did": "string"
  }
}
```

#### Example
```json
{
  "delete_service": {
    "did": "did:c4e:alice",
    "service_did": "did:c4e:alice#storage"
  }
}
```

#### Response Events
```json
{
  "type": "wasm-delete_service",
  "attributes": [
    {"key": "did", "value": "did:c4e:alice"},
    {"key": "deleted_service", "value": "did:c4e:alice#storage"}
  ]
}
```

### 7. Delete DID Document

Completely removes a DID document from the contract.

#### Schema
```json
{
  "delete_did_document": {
    "did": "string"
  }
}
```

#### Authorization
- Caller must be a controller of the target DID

#### Example
```json
{
  "delete_did_document": {
    "did": "did:c4e:alice"
  }
}
```

#### Response Events
```json
{
  "type": "wasm-delete_did_document",
  "attributes": [
    {"key": "did", "value": "did:c4e:alice"},
    {"key": "deleted_controllers", "value": "c4e1abc...def,c4e1xyz...123"},
    {"key": "deleted_services", "value": "did:c4e:alice#messaging,did:c4e:alice#auth"}
  ]
}
```

## Query Messages

### 1. Get DID Document

Retrieves a complete DID document by its identifier.

#### Schema
```json
{
  "get_did_document": {
    "did": "string"
  }
}
```

#### Response
```json
{
  "id": "did:c4e:alice",
  "controller": ["c4e1abc...def", "c4e1xyz...123"],
  "service": [
    {
      "id": "did:c4e:alice#messaging",
      "a_type": "MessagingService",
      "service_endpoint": "https://alice.example.com/messaging"
    }
  ]
}
```

#### Example Query
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "get_did_document": {
    "did": "did:c4e:alice"
  }
}'
```

### 2. Check DID Controller

Verifies if an entity is a controller of a specific DID.

#### Schema
```json
{
  "is_did_controller": {
    "did": "string",
    "controller": "string"
  }
}
```

#### Response
```json
true
```

#### Example
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "is_did_controller": {
    "did": "did:c4e:alice",
    "controller": "c4e1abc...def"
  }
}'
```

### 3. Check Controller Relationship

Verifies if a controller has authority over any DID in a list.

#### Schema
```json
{
  "is_controller_of": {
    "controllers": ["string"],
    "controller": "string"
  }
}
```

#### Response
```json
true
```

### 4. Check Controllers Existence

Verifies if all controllers in a list exist as DIDs or addresses.

#### Schema
```json
{
  "do_controllers_exist": {
    "controllers": ["string"]
  }
}
```

#### Response
```json
true
```

### 5. Check Single Controller Existence

Verifies if a single controller exists.

#### Schema
```json
{
  "does_controller_exist": {
    "controller": "string"
  }
}
```

#### Response
```json
true
```

### 6. Get Controlled DIDs

Returns all DIDs controlled by a specific controller.

#### Schema
```json
{
  "get_controlled_dids": {
    "controller": "string",
    "limit": 30,
    "start_after": "string"
  }
}
```

#### Parameters
- `controller` (string): Controller address or DID
- `limit` (optional, number): Maximum results (default: 30, max: 100)
- `start_after` (optional, string): Pagination cursor

#### Response
```json
[
  "did:c4e:alice",
  "did:c4e:bob",
  "did:c4e:organization"
]
```

#### Example
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "get_controlled_dids": {
    "controller": "c4e1abc...def",
    "limit": 10
  }
}'
```

### 7. Get Controlled DID Documents

Returns complete DID documents for all DIDs controlled by a controller.

#### Schema
```json
{
  "get_controlled_did_documents": {
    "controller": "string",
    "limit": 30,
    "start_after": "string"
  }
}
```

#### Response
```json
[
  {
    "id": "did:c4e:alice",
    "controller": ["c4e1abc...def"],
    "service": [
      {
        "id": "did:c4e:alice#messaging",
        "a_type": "MessagingService",
        "service_endpoint": "https://alice.example.com/messaging"
      }
    ]
  }
]
```

## Error Codes

### Validation Errors
- `DidFormatError`: Invalid DID format (must start with `did:c4e:`)
- `InvalidAddress`: Invalid Cosmos address format
- `DuplicatedController`: Duplicate controllers in the same document
- `DuplicatedService`: Duplicate services in the same document

### Authorization Errors
- `Unauthorized`: Caller is not authorized to perform the action
- `DidDocumentNotFound`: Referenced DID document does not exist
- `ControllerNotFound`: Referenced controller does not exist

### Business Logic Errors
- `DidDocumentAlreadyExists`: Cannot create DID that already exists
- `DidDocumentNoController`: DID document must have at least one controller
- `SelfControlledDidDocumentNotAllowed`: DID cannot control itself
- `ServiceNotFound`: Referenced service does not exist in the document
- `NoControllers`: Controllers list cannot be empty

## Data Types

### DID Document
```json
{
  "id": "string",           // DID identifier (did:c4e:...)
  "controller": ["string"], // Array of controller addresses/DIDs
  "service": [              // Array of service endpoints
    {
      "id": "string",           // Service DID identifier
      "a_type": "string",       // Service type (MessagingService, etc.)
      "service_endpoint": "string" // Service URL endpoint
    }
  ]
}
```

### Service
```json
{
  "id": "string",           // Service identifier (DID format)
  "a_type": "string",       // Service type classification
  "service_endpoint": "string" // HTTP/HTTPS endpoint URL
}
```

## Usage Examples

### Complete Workflow Example

#### 1. Create Organization DID
```bash
c4ed tx wasm execute <contract_addr> '{
  "create_did_document": {
    "did_doc": {
      "id": "did:c4e:acme-corp",
      "controller": ["c4e1admin...address"],
      "service": [
        {
          "id": "did:c4e:acme-corp#website",
          "a_type": "WebsiteService",
          "service_endpoint": "https://acme-corp.example.com"
        }
      ]
    }
  }
}' --from admin
```

#### 2. Create Employee DID Controlled by Organization
```bash
c4ed tx wasm execute <contract_addr> '{
  "create_did_document": {
    "did_doc": {
      "id": "did:c4e:employee-alice",
      "controller": ["did:c4e:acme-corp", "c4e1alice...address"],
      "service": [
        {
          "id": "did:c4e:employee-alice#profile",
          "a_type": "ProfileService", 
          "service_endpoint": "https://acme-corp.example.com/employees/alice"
        }
      ]
    }
  }
}' --from admin
```

#### 3. Add New Service to Employee
```bash
c4ed tx wasm execute <contract_addr> '{
  "add_service": {
    "did": "did:c4e:employee-alice",
    "service": {
      "id": "did:c4e:employee-alice#email",
      "a_type": "EmailService",
      "service_endpoint": "mailto:alice@acme-corp.example.com"
    }
  }
}' --from alice
```

#### 4. Query Employee's DID Document
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "get_did_document": {
    "did": "did:c4e:employee-alice"
  }
}'
```

#### 5. List All Organization-Controlled DIDs
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "get_controlled_dids": {
    "controller": "did:c4e:acme-corp",
    "limit": 50
  }
}'
```

### Identity Verification Example

#### 1. Verify Authorization
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "is_did_controller": {
    "did": "did:c4e:employee-alice",
    "controller": "did:c4e:acme-corp"
  }
}'
```

#### 2. Check Multiple Controllers
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "do_controllers_exist": {
    "controllers": ["did:c4e:acme-corp", "c4e1alice...address"]
  }
}'
```

## Integration Patterns

### 1. Identity Verification Service
```typescript
class IdentityVerifier {
  async verifyControl(did: string, controller: string): Promise<boolean> {
    const query = {
      is_did_controller: { did, controller }
    };
    return await this.client.queryContractSmart(this.contractAddress, query);
  }
  
  async getControlledIdentities(controller: string): Promise<string[]> {
    const query = {
      get_controlled_dids: { controller, limit: 100 }
    };
    return await this.client.queryContractSmart(this.contractAddress, query);
  }
}
```

### 2. Organization Management
```typescript
class OrganizationManager {
  async createEmployeeDID(orgDID: string, employeeAddress: string, employeeId: string): Promise<void> {
    const executeMsg = {
      create_did_document: {
        did_doc: {
          id: `did:c4e:${orgDID}-employee-${employeeId}`,
          controller: [orgDID, employeeAddress],
          service: [
            {
              id: `did:c4e:${orgDID}-employee-${employeeId}#profile`,
              a_type: "EmployeeProfile",
              service_endpoint: `https://org.example.com/employees/${employeeId}`
            }
          ]
        }
      }
    };
    
    await this.client.execute(this.senderAddress, this.contractAddress, executeMsg, "auto");
  }
}
```

### 3. Service Discovery
```typescript
class ServiceDiscovery {
  async findServices(did: string, serviceType: string): Promise<Service[]> {
    const didDoc = await this.getDIDDocument(did);
    return didDoc.service.filter(s => s.a_type === serviceType);
  }
  
  async getDIDDocument(did: string): Promise<DIDDocument> {
    const query = { get_did_document: { did } };
    return await this.client.queryContractSmart(this.contractAddress, query);
  }
}
```