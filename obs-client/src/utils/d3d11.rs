use crate::error::ObsError;
use std::ptr;
use winapi::{
    shared::winerror::FAILED,
    um::{
        d3d11::{D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, D3D11_SDK_VERSION},
        d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
    },
    Interface,
};
use wio::com::ComPtr;

pub fn create_device() -> Result<(ComPtr<ID3D11Device>, ComPtr<ID3D11DeviceContext>), ObsError> {
    let mut device: *mut ID3D11Device = ptr::null_mut();
    let mut device_context: *mut ID3D11DeviceContext = ptr::null_mut();

    log::info!("Creating the device");
    let result = unsafe {
        D3D11CreateDevice(
            ptr::null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            0,
            D3D11_SDK_VERSION,
            &mut device,
            ptr::null_mut(),
            &mut device_context,
        )
    };
    if FAILED(result) {
        log::error!("Failed to create device");
        return Err(ObsError::CreateDevice);
    }
    let device = unsafe { ComPtr::from_raw(device) };
    let device_context = unsafe { ComPtr::from_raw(device_context) };

    Ok((device, device_context))
}

pub fn open_resource(device: &ComPtr<ID3D11Device>, handle: u32) -> Result<ComPtr<ID3D11Resource>, ObsError> {
    log::info!("Opening the shared resource");
    let mut resource: *mut ID3D11Resource = ptr::null_mut();
    let result = unsafe {
        device.OpenSharedResource(
            handle as _,
            &ID3D11Resource::uuidof(),
            &mut resource as *mut *mut _ as _,
        )
    };
    if FAILED(result) {
        log::error!("Failed to open the shared resource");
        return Err(ObsError::OpenSharedResource);
    }
    let resource = unsafe { ComPtr::from_raw(resource) };

    Ok(resource)
}
