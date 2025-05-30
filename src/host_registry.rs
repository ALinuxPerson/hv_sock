use std::{fmt, mem};
use std::fmt::Formatter;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use getset::Getters;
use uuid::Uuid;
use windows::core::GUID;
use windows::Win32::System::Hypervisor::{HV_GUID_BROADCAST, HV_GUID_CHILDREN, HV_GUID_LOOPBACK, HV_GUID_PARENT, HV_GUID_SILOHOST, HV_GUID_VSOCK_TEMPLATE, HV_GUID_ZERO};
use windows_registry::{Key, KeyIterator};

pub const HIVE: &Key = windows_registry::LOCAL_MACHINE;
pub const KEY: &'static str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Virtualization\\GuestCommunicationServices";
pub const ELEMENT_NAME: &'static str = "ElementName";

/// Represents the GUID_ZERO {00000000-0000-0000-0000-000000000000}.
pub const ZERO: Uuid = guid_to_uuid(HV_GUID_ZERO);
/// Represents the GUID_WILDCARD {00000000-0000-0000-0000-000000000000}. This is an alias for `ZERO`.
pub const WILDCARD: Uuid = ZERO;
/// Represents the GUID_BROADCAST {FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF}.
pub const BROADCAST: Uuid = guid_to_uuid(HV_GUID_BROADCAST);
/// Represents the GUID_CHILDREN {90db8b89-0d35-4f79-8ce9-49ea0ac8b7cd}.
pub const CHILDREN: Uuid = guid_to_uuid(HV_GUID_CHILDREN);
/// Represents the GUID_LOOPBACK {e0e16197-dd56-4a10-9195-5ee7a155a838}.
pub const LOOPBACK: Uuid = guid_to_uuid(HV_GUID_LOOPBACK);
/// Represents the GUID_PARENT {a42e7cda-d03f-480c-9cc2-a4de20abb878}.
pub const PARENT: Uuid = guid_to_uuid(HV_GUID_PARENT);
/// Represents the GUID_VSOCK_TEMPLATE {00000000-facb-11e6-bd58-64006a7986d3}.
pub const VSOCK_TEMPLATE: Uuid = guid_to_uuid(HV_GUID_VSOCK_TEMPLATE);
/// Represents the GUID_SILOHOST {36bd0c5c-7276-4223-88ba-7d03b654c568}.
pub const SILO_HOST: Uuid = guid_to_uuid(HV_GUID_SILOHOST); // what's this?

const fn guid_to_uuid(GUID { data1, data2, data3, data4 }: GUID) -> Uuid {
    Uuid::from_fields(data1, data2, data3, &data4)
}

const fn uuid_as_fields(uuid: &Uuid) -> (u32, u16, u16, [u8; 8]) {
    let bytes = uuid.as_bytes();

    let d1 = (bytes[0] as u32) << 24
        | (bytes[1] as u32) << 16
        | (bytes[2] as u32) << 8
        | (bytes[3] as u32);

    let d2 = (bytes[4] as u16) << 8 | (bytes[5] as u16);

    let d3 = (bytes[6] as u16) << 8 | (bytes[7] as u16);

    let d4: [u8; 8] = [
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    ];
    (d1, d2, d3, d4)
}

const fn uuid_eq(lhs: &Uuid, rhs: &Uuid) -> bool {
    let (ld1, ld2, ld3, [ld4_0, ld4_1, ld4_2, ld4_3, ld4_4, ld4_5, ld4_6, ld4_7]) = uuid_as_fields(lhs);
    let (rd1, rd2, rd3, [rd4_0, rd4_1, rd4_2, rd4_3, rd4_4, rd4_5, rd4_6, rd4_7]) = uuid_as_fields(rhs);

    ld1 == rd1 && ld2 == rd2 && ld3 == rd3 && ld4_0 == rd4_0 && ld4_1 == rd4_1
        && ld4_2 == rd4_2 && ld4_3 == rd4_3 && ld4_4 == rd4_4 && ld4_5 == rd4_5
        && ld4_6 == rd4_6 && ld4_7 == rd4_7
}

/// Represents a service UUID, which can be either a Windows GUID or a Linux AF_VSOCK port.
#[derive(Copy, Clone)]
pub enum ServiceUuidRepr {
    /// A Windows-style service GUID.
    Windows(Uuid),
    /// A Linux-style AF_VSOCK port.
    Linux { port: u32 },
}

/// A wrapper around `ServiceUuidRepr` to provide convenient methods for working with service UUIDs.
#[derive(Copy, Clone)]
pub struct ServiceUuid(ServiceUuidRepr);

impl ServiceUuid {
    /// The zero service UUID.
    pub const ZERO: Self = Self::from_uuid(ZERO);
    /// The wildcard service UUID.
    pub const WILDCARD: Self = Self::from_uuid(WILDCARD);
    /// The broadcast service UUID.
    pub const BROADCAST: Self = Self::from_uuid(BROADCAST);
    /// The children service UUID.
    pub const CHILDREN: Self = Self::from_uuid(CHILDREN);
    /// The loopback service UUID.
    pub const LOOPBACK: Self = Self::from_uuid(LOOPBACK);
    /// The parent service UUID.
    pub const PARENT: Self = Self::from_uuid(PARENT);
    /// The VSOCK template service UUID.
    pub const VSOCK_TEMPLATE: Self = Self::from_uuid(VSOCK_TEMPLATE);
    /// The silo host service UUID.
    pub const SILO_HOST: Self = Self::from_uuid(SILO_HOST);

    /// Creates a `ServiceUuid` from a `Uuid`.
    ///
    /// If the `Uuid` matches the `VSOCK_TEMPLATE` (ignoring the first data field),
    /// it is interpreted as a Linux AF_VSOCK port, where the first data field of the `Uuid`
    /// is the port number. Otherwise, it is interpreted as a Windows GUID.
    pub const fn from_uuid(uuid: Uuid) -> Self {
        let (d1, d2, d3, d4) = uuid_as_fields(&uuid);

        if uuid_eq(&Uuid::from_fields(0, d2, d3, &d4), &VSOCK_TEMPLATE) {
            Self(ServiceUuidRepr::Linux { port: d1 })
        } else {
            Self(ServiceUuidRepr::Windows(uuid))
        }
    }

    /// Creates a `ServiceUuid` representing a Windows GUID.
    ///
    /// Returns `None` if the provided `Uuid` would be interpreted as a Linux AF_VSOCK port.
    pub const fn windows(uuid: Uuid) -> Option<Self> {
        let (_, d2, d3, d4) = uuid_as_fields(&uuid);
        if uuid_eq(&Uuid::from_fields(0, d2, d3, &d4), &VSOCK_TEMPLATE) {
            None
        } else {
            Some(Self(ServiceUuidRepr::Windows(uuid)))
        }
    }

    /// Creates a `ServiceUuid` representing a Linux AF_VSOCK port.
    pub const fn linux(port: u32) -> Self {
        Self(ServiceUuidRepr::Linux { port })
    }

    /// Returns the underlying `ServiceUuidRepr`.
    pub const fn repr(&self) -> ServiceUuidRepr {
        self.0
    }

    /// Renders the `ServiceUuid` as a `Uuid`.
    ///
    /// For Linux AF_VSOCK ports, this constructs a `Uuid` using the `VSOCK_TEMPLATE`
    /// and the port number. For Windows GUIDs, it returns the underlying `Uuid`.
    pub const fn render(&self) -> Uuid {
        match self.0 {
            ServiceUuidRepr::Windows(uuid) => uuid,
            ServiceUuidRepr::Linux { port } => {
                let (_, d2, d3, d4) = uuid_as_fields(&VSOCK_TEMPLATE);
                Uuid::from_fields(port, d2, d3, &d4)
            },
        }
    }
}

impl From<Uuid> for ServiceUuid {
    fn from(value: Uuid) -> Self {
        Self::from_uuid(value)
    }
}

impl From<ServiceUuid> for Uuid {
    fn from(value: ServiceUuid) -> Self {
        value.render()
    }
}

impl fmt::Display for ServiceUuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.render().fmt(f)
    }
}

/// Data associated with a registered service.
pub struct ServiceData {
    /// The service UUID.
    pub uuid: ServiceUuid,
    /// The human-readable element name of the service.
    pub element_name: String,
}

/// Represents a registered Hyper-V service.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct Service {
    data: ServiceData,
    key: Key,
}

impl Service {
    /// Sets the element name of the service in the registry.
    ///
    /// Returns the old element name.
    pub fn set_element_name(&mut self, to: String) -> windows_registry::Result<String> {
        self.key.set_string(ELEMENT_NAME, &to)?;
        Ok(mem::replace(&mut self.data.element_name, to))
    }
}

/// Provides access to the Hyper-V host service registry.
///
/// This registry stores information about services available for guest communication.
pub struct HostRegistry {
    key: Key,
    lock: Option<RwLock<()>>,
}

impl HostRegistry {
    fn from_key(key: Key) -> Self {
        Self { key, lock: Some(RwLock::new(())) }
    }

    /// Opens the host service registry.
    ///
    /// This method uses a `RwLock` to synchronize access to the registry.
    pub fn open() -> windows_registry::Result<Self> {
        Ok(Self::from_key(HIVE.open(KEY)?))
    }

    /// Creates the host service registry if it doesn't exist.
    ///
    /// This method uses a `RwLock` to synchronize access to the registry.
    pub fn create() -> windows_registry::Result<Self> {
        Ok(Self::from_key(HIVE.create(KEY)?))
    }
}

impl HostRegistry {
    fn from_key_no_lock(key: Key) -> Self {
        Self { key, lock: None }
    }

    /// Opens the host service registry without locking.
    ///
    /// **Warning:** This method does not provide any synchronization.
    /// Concurrent access may lead to race conditions.
    pub fn open_no_lock() -> windows_registry::Result<Self> {
        Ok(Self::from_key_no_lock(HIVE.open(KEY)?))
    }

    /// Creates the host service registry if it doesn't exist, without locking.
    ///
    /// **Warning:** This method does not provide any synchronization.
    /// Concurrent access may lead to race conditions.
    pub fn create_no_lock() -> windows_registry::Result<Self> {
        Ok(Self::from_key_no_lock(HIVE.create(KEY)?))
    }
}

impl HostRegistry {
    /// Returns a reference to the underlying registry key.
    pub fn key(&self) -> &Key {
        &self.key
    }
}

impl HostRegistry {
    /// Enables or disables locking for registry access.
    ///
    /// If locking is enabled, a `RwLock` is used to synchronize access.
    /// If locking is disabled, the `RwLock` is removed.
    pub fn lock(&mut self, lock: bool) {
        if lock {
            if self.lock.is_none() {
                self.lock = Some(RwLock::new(()))
            }
        } else {
            self.lock = None
        }
    }

    fn read(&self) -> Option<RwLockReadGuard<()>> {
        self.lock.as_ref().map(|l| l.read().unwrap())
    }

    fn read_with<R>(&self, f: impl FnOnce(&Key) -> R) -> R {
        let _guard = self.read();
        f(&self.key)
    }

    fn write(&self) -> Option<RwLockWriteGuard<()>> {
        self.lock.as_ref().map(|l| l.write().unwrap())
    }

    fn write_with<R>(&self, f: impl FnOnce(&Key) -> R) -> R {
        let _guard = self.write();
        f(&self.key)
    }
}

impl HostRegistry {
    /// Registers a new service in the host registry.
    pub fn register(&self, service: ServiceData) -> windows_registry::Result<Service> {
        let key = self.key.create(&service.uuid.to_string())?;
        self.write_with(|key| key.set_string(ELEMENT_NAME, &service.element_name))?;
        Ok(Service { data: service, key })
    }

    /// Deletes a service from the host registry.
    pub fn delete(&self, uuid: ServiceUuid) -> windows_registry::Result<()> {
        self.write_with(|key| key.remove_tree(&uuid.to_string()))?;
        Ok(())
    }

    /// Retrieves a service from the host registry.
    pub fn get(&self, uuid: ServiceUuid) -> windows_registry::Result<Service> {
        let key = self.read_with(|key| key.open(&uuid.to_string()))?;
        let element_name = key.get_string(ELEMENT_NAME)?;

        Ok(Service { data: ServiceData { uuid, element_name }, key })
    }

    /// Renames a service in the host registry.
    ///
    /// This is equivalent to getting the service data, deleting the old entry,
    /// and registering a new entry with the new UUID and the old data.
    pub fn rename(&self, from: ServiceUuid, to: ServiceUuid) -> windows_registry::Result<Service> {
        let element_name = self.get(from)?.data.element_name;
        self.delete(from)?;
        self.register(ServiceData { uuid: to, element_name })
    }
}

impl HostRegistry {
    /// Returns an iterator over the services in the host registry.
    pub fn iter(&self) -> windows_registry::Result<Iter> {
        Ok(Iter { host_registry: self, keys: self.key.keys()? })
    }
}

/// An iterator over the services in the host registry.
pub struct Iter<'hr> {
    host_registry: &'hr HostRegistry,
    keys: KeyIterator<'hr>,
}

impl<'hr> Iterator for Iter<'hr> {
    type Item = windows_registry::Result<Service>;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().map(|k| self.host_registry.get(ServiceUuid::from_uuid(k.parse().unwrap())))
    }
}
