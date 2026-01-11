use anyhow::Result;
use uc_core::device::DeviceId;
use uc_core::ports::DeviceIdentityPort;

pub struct LocalDeviceIdentity {
    device_id: DeviceId,
}

impl LocalDeviceIdentity {
    pub fn load_or_create() -> Result<Self> {
        if let Some(id) = load_from_disk()? {
            Ok(Self { device_id: id })
        } else {
            let id = DeviceId::new(uuid::Uuid::new_v4().to_string());
            save_to_disk(&id)?;
            Ok(Self { device_id: id })
        }
    }
}

impl DeviceIdentityPort for LocalDeviceIdentity {
    fn current_device_id(&self) -> DeviceId {
        self.device_id.clone()
    }
}
