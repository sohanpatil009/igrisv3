# Requirements Document

## Introduction

This document specifies the requirements for enhancing the trusted devices management system in IGRIS, a Rust-based file sharing application. The enhancements add bulk operations, trust groups, history tracking, advanced security features, UI improvements, and notifications to the existing trusted devices system while maintaining backward compatibility and security guarantees.

## Glossary

- **Trust_Manager**: The component responsible for establishing, verifying, and managing trust relationships between devices
- **Device_Config**: The persistent configuration containing device identity and trusted devices list
- **Trusted_Device**: A device that has been verified and authorized to connect
- **Trust_Group**: A user-defined category for organizing trusted devices
- **Trust_History**: A log of connection attempts, trust changes, and device interactions
- **Certificate_Fingerprint**: SHA-256 hash of a device's TLS certificate used for identification
- **Rate_Limiter**: Security mechanism that blocks connection attempts after repeated failures
- **Device_Identity**: The local device's identifying information (ID, label, OS)
- **Connection_Log**: Record of connection attempts with timestamps and outcomes
- **Trust_Audit_Trail**: Chronological record of trust establishment and modification events

## Requirements

### Requirement 1: Bulk Device Selection and Operations

**User Story:** As a user managing many trusted devices, I want to select multiple devices at once and perform batch operations, so that I can efficiently manage my device list.

#### Acceptance Criteria

1. WHEN a user enters selection mode, THE UI SHALL display checkboxes next to each trusted device
2. WHEN a user selects multiple devices, THE UI SHALL display the count of selected devices
3. WHEN a user clicks "Remove Selected", THE Trust_Manager SHALL remove all selected devices from the trusted list
4. WHEN a user clicks "Export Selected", THE Device_Config SHALL serialize selected devices to JSON format
5. WHEN a user provides a JSON file for import, THE Device_Config SHALL validate and add devices to the trusted list
6. WHEN a user applies a bulk rename pattern, THE Trust_Manager SHALL update device labels according to the pattern
7. IF a bulk operation fails for any device, THEN THE Trust_Manager SHALL continue processing remaining devices and report failures

### Requirement 2: Trust Groups and Categories

**User Story:** As a user with devices in different contexts, I want to organize devices into groups, so that I can manage permissions and access by category.

#### Acceptance Criteria

1. THE Device_Config SHALL store group assignments for each trusted device
2. WHEN a user creates a new group, THE Trust_Manager SHALL validate the group name is unique and non-empty
3. WHEN a user assigns a device to a group, THE Device_Config SHALL persist the group assignment
4. WHEN a user filters by group, THE UI SHALL display only devices in that group
5. WHERE a device belongs to a group with auto-accept enabled, THE Trust_Manager SHALL automatically accept connection requests from that device
6. WHEN a user deletes a group, THE Trust_Manager SHALL move all devices in that group to the default "Ungrouped" category
7. THE UI SHALL display visual indicators showing each device's group membership

### Requirement 3: Connection History Logging

**User Story:** As a security-conscious user, I want to see a history of all connection attempts and trust changes, so that I can audit device activity.

#### Acceptance Criteria

1. WHEN a connection attempt occurs, THE Trust_Manager SHALL log the timestamp, device ID, IP address, and outcome
2. WHEN a trust relationship is established or modified, THE Trust_Manager SHALL record the event in the audit trail
3. WHEN a connection fails, THE Trust_Manager SHALL log the failure reason and increment the failure count
4. WHEN a user views device history, THE UI SHALL display connection logs in reverse chronological order
5. WHEN a user exports history, THE Trust_Manager SHALL serialize logs to CSV or JSON format
6. THE Connection_Log SHALL retain entries for at least 90 days
7. WHEN log storage exceeds 10MB, THE Trust_Manager SHALL archive oldest entries

### Requirement 4: Advanced Security Configuration

**User Story:** As a user requiring enhanced security, I want to configure custom trust expiry periods and approval modes, so that I can control device access more precisely.

#### Acceptance Criteria

1. WHERE a device has a custom expiry period, THE Trust_Manager SHALL use that period instead of the default 30 days
2. WHERE manual approval mode is enabled, THE Trust_Manager SHALL require explicit user confirmation before trusting new devices
3. WHEN a user adds notes to a device, THE Device_Config SHALL persist the notes with the device record
4. WHEN a connection occurs, THE Trust_Manager SHALL record the device's IP address
5. THE Trust_Manager SHALL track connection statistics including total transfers and data volume per device
6. WHEN suspicious activity is detected, THE Trust_Manager SHALL generate a security alert
7. IF a device attempts connection from a different IP than previously recorded, THEN THE Trust_Manager SHALL flag this as potentially suspicious

### Requirement 5: Enhanced Device Search and Filtering

**User Story:** As a user with many trusted devices, I want to search and filter my device list, so that I can quickly find specific devices.

#### Acceptance Criteria

1. WHEN a user enters a search query, THE UI SHALL filter devices matching the query in name, ID, or notes
2. WHEN a user selects a sort option, THE UI SHALL reorder devices by the selected criterion
3. THE UI SHALL support sorting by device name, last connected time, trust date, and group
4. WHEN a user clicks a device, THE UI SHALL display a modal with complete device information
5. THE UI SHALL display connection status indicators showing online, offline, or connecting states
6. WHEN a user right-clicks a device, THE UI SHALL show a context menu with quick actions
7. WHEN a user drags a device onto a group, THE Trust_Manager SHALL assign the device to that group

### Requirement 6: Trust Expiry and Connection Notifications

**User Story:** As a user who wants to stay informed, I want to receive notifications about trust expiry and connection events, so that I can maintain security awareness.

#### Acceptance Criteria

1. WHEN a device trust will expire within 7 days, THE Trust_Manager SHALL generate an expiry warning notification
2. WHEN a new device connects for the first time, THE Trust_Manager SHALL generate a new device notification
3. WHEN a connection attempt fails repeatedly, THE Trust_Manager SHALL generate a failed attempt alert
4. WHEN the rate limiter blocks a device, THE Trust_Manager SHALL generate a rate limit notification
5. THE Device_Config SHALL store user notification preferences per notification type
6. WHEN a notification is generated, THE UI SHALL display it according to user preferences
7. THE Trust_Manager SHALL not generate duplicate notifications for the same event within 1 hour

### Requirement 7: Data Persistence and Migration

**User Story:** As a user upgrading from the current version, I want my existing trusted devices to work seamlessly, so that I don't lose my trust relationships.

#### Acceptance Criteria

1. WHEN the application loads an old format file_share.json, THE Device_Config SHALL migrate it to the new format
2. THE Device_Config SHALL add default values for new fields when migrating existing devices
3. WHEN migration completes, THE Device_Config SHALL preserve all existing device data
4. THE Device_Config SHALL maintain backward compatibility with the existing JSON schema
5. IF migration fails, THEN THE Device_Config SHALL preserve the original file and log the error
6. THE Device_Config SHALL validate JSON structure before attempting to load it
7. WHEN saving configuration, THE Device_Config SHALL write atomically to prevent corruption

### Requirement 8: Performance and Scalability

**User Story:** As a user with many trusted devices, I want the system to remain responsive, so that device management doesn't slow down my workflow.

#### Acceptance Criteria

1. WHEN loading 100+ trusted devices, THE UI SHALL render the device list within 500ms
2. WHEN searching devices, THE UI SHALL update results within 100ms of keystroke
3. THE Trust_Manager SHALL use indexed lookups for device verification
4. WHEN filtering by group, THE UI SHALL apply filters without blocking the main thread
5. THE Connection_Log SHALL use efficient storage to handle 10,000+ log entries
6. WHEN exporting large datasets, THE Trust_Manager SHALL perform the operation asynchronously
7. THE Device_Config SHALL load configuration data lazily to minimize startup time

### Requirement 9: JSON Import and Export Validation

**User Story:** As a user sharing device configurations, I want to safely import and export device lists, so that I can transfer trust relationships between installations.

#### Acceptance Criteria

1. WHEN exporting devices, THE Device_Config SHALL include all device fields in valid JSON format
2. WHEN importing devices, THE Device_Config SHALL validate JSON schema before processing
3. IF imported JSON contains invalid device data, THEN THE Device_Config SHALL reject the import and report specific errors
4. WHEN importing devices with duplicate IDs, THE Trust_Manager SHALL prompt the user to resolve conflicts
5. THE Device_Config SHALL validate certificate fingerprints are valid SHA-256 hashes
6. WHEN importing devices, THE Trust_Manager SHALL verify timestamps are valid ISO 8601 format
7. THE Device_Config SHALL sanitize imported device labels to prevent injection attacks

### Requirement 10: Connection Status Tracking

**User Story:** As a user monitoring my network, I want to see which trusted devices are currently online, so that I know who can access my files.

#### Acceptance Criteria

1. WHEN a device connects successfully, THE Trust_Manager SHALL update the device status to "online"
2. WHEN a device disconnects, THE Trust_Manager SHALL update the device status to "offline"
3. WHEN a connection is in progress, THE Trust_Manager SHALL update the device status to "connecting"
4. THE UI SHALL poll connection status every 5 seconds and update indicators
5. WHEN a device has been offline for 24 hours, THE UI SHALL display the last connected timestamp
6. THE Trust_Manager SHALL maintain connection state in memory without persisting to disk
7. WHEN the application starts, THE Trust_Manager SHALL initialize all devices to "offline" status
