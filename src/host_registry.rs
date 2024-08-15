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

pub const ZERO: Uuid = guid_to_uuid(HV_GUID_ZERO);
pub const WILDCARD: Uuid = ZERO;
pub const BROADCAST: Uuid = guid_to_uuid(HV_GUID_BROADCAST);
pub const CHILDREN: Uuid = guid_to_uuid(HV_GUID_CHILDREN);
pub const LOOPBACK: Uuid = guid_to_uuid(HV_GUID_LOOPBACK);
pub const PARENT: Uuid = guid_to_uuid(HV_GUID_PARENT);
pub const VSOCK_TEMPLATE: Uuid = guid_to_uuid(HV_GUID_VSOCK_TEMPLATE);
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

#[derive(Copy, Clone)]
pub enum ServiceUuidRepr {
    Windows(Uuid),
    Linux { port: u32 },
}

#[derive(Copy, Clone)]
pub struct ServiceUuid(ServiceUuidRepr);

impl ServiceUuid {
    pub const ZERO: Self = Self::from_uuid(ZERO);
    pub const WILDCARD: Self = Self::from_uuid(WILDCARD);
    pub const BROADCAST: Self = Self::from_uuid(BROADCAST);
    pub const CHILDREN: Self = Self::from_uuid(CHILDREN);
    pub const LOOPBACK: Self = Self::from_uuid(LOOPBACK);
    pub const PARENT: Self = Self::from_uuid(PARENT);
    pub const VSOCK_TEMPLATE: Self = Self::from_uuid(VSOCK_TEMPLATE);
    pub const SILO_HOST: Self = Self::from_uuid(SILO_HOST);

    pub const fn from_uuid(uuid: Uuid) -> Self {
        let (d1, d2, d3, d4) = uuid_as_fields(&uuid);

        if uuid_eq(&Uuid::from_fields(0, d2, d3, &d4), &VSOCK_TEMPLATE) {
            Self(ServiceUuidRepr::Linux { port: d1 })
        } else {
            Self(ServiceUuidRepr::Windows(uuid))
        }
    }

    pub const fn windows(uuid: Uuid) -> Option<Self> {
        let (_, d2, d3, d4) = uuid_as_fields(&uuid);
        if uuid_eq(&Uuid::from_fields(0, d2, d3, &d4), &VSOCK_TEMPLATE) {
            None
        } else {
            Some(Self(ServiceUuidRepr::Windows(uuid)))
        }
    }

    pub const fn linux(port: u32) -> Self {
        Self(ServiceUuidRepr::Linux { port })
    }

    pub const fn repr(&self) -> ServiceUuidRepr {
        self.0
    }

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

pub struct ServiceData {
    pub uuid: ServiceUuid,
    pub element_name: String,
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct Service {
    data: ServiceData,
    key: Key,
}

impl Service {
    pub fn set_element_name(&mut self, to: String) -> windows_registry::Result<String> {
        self.key.set_string(ELEMENT_NAME, &to)?;
        Ok(mem::replace(&mut self.data.element_name, to))
    }
}

pub struct HostRegistry {
    key: Key,
    lock: Option<RwLock<()>>,
}

impl HostRegistry {
    fn from_key(key: Key) -> Self {
        Self { key, lock: Some(RwLock::new(())) }
    }

    pub fn open() -> windows_registry::Result<Self> {
        Ok(Self::from_key(HIVE.open(KEY)?))
    }

    pub fn create() -> windows_registry::Result<Self> {
        Ok(Self::from_key(HIVE.create(KEY)?))
    }
}

impl HostRegistry {
    fn from_key_no_lock(key: Key) -> Self {
        Self { key, lock: None }
    }

    pub fn open_no_lock() -> windows_registry::Result<Self> {
        Ok(Self::from_key_no_lock(HIVE.open(KEY)?))
    }

    pub fn create_no_lock() -> windows_registry::Result<Self> {
        Ok(Self::from_key_no_lock(HIVE.create(KEY)?))
    }
}

impl HostRegistry {
    pub fn key(&self) -> &Key {
        &self.key
    }
}

impl HostRegistry {
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
    pub fn register(&self, service: ServiceData) -> windows_registry::Result<Service> {
        let key = self.key.create(&service.uuid.to_string())?;
        self.write_with(|key| key.set_string(ELEMENT_NAME, &service.element_name))?;
        Ok(Service { data: service, key })
    }

    pub fn delete(&self, uuid: ServiceUuid) -> windows_registry::Result<()> {
        self.write_with(|key| key.remove_tree(&uuid.to_string()))?;
        Ok(())
    }

    pub fn get(&self, uuid: ServiceUuid) -> windows_registry::Result<Service> {
        let key = self.read_with(|key| key.open(&uuid.to_string()))?;
        let element_name = key.get_string(ELEMENT_NAME)?;

        Ok(Service { data: ServiceData { uuid, element_name }, key })
    }

    pub fn rename(&self, from: ServiceUuid, to: ServiceUuid) -> windows_registry::Result<Service> {
        let element_name = self.get(from)?.data.element_name;
        self.delete(from)?;
        self.register(ServiceData { uuid: to, element_name })
    }
}

impl HostRegistry {
    pub fn iter(&self) -> windows_registry::Result<Iter> {
        Ok(Iter { host_registry: self, keys: self.key.keys()? })
    }
}

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
