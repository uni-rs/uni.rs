//! Implementation of Xen's network driver

use core::mem;

use sync::Arc;
use boxed::Box;

use vec::Vec;
use string::ToString;

use sync::spin::{InterruptSpinLock, SpinLock, RwLock};

use ffi::CString;

use net::{Interface, Packet};
use net::defs::HwAddr;

use hal::xen::store::{XenStore, XenbusState};
use hal::xen::grant::Ref as GrantRef;
use hal::xen::ring::{SharedRing, FrontRing};

use hal::xen::defs::EvtchnPort;

use hal::arch::defs::PAGE_SIZE;

use hal::xen::event;

#[repr(i16)]
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
/// XEN_NETIF_RSP_*
enum NetifRsp {
    /// XEN_NETIF_RSP_OKAY
    Okay = 0,
    /// XEN_NETIF_RSP_ERROR
    Error = -1,
    /// XEN_NETIF_RSP_DROPPED
    Dropped = -2,
    /// XEN_NETIF_RSP_NULL
    Null = 1,
}

#[repr(u16)]
#[derive(Debug)]
#[allow(dead_code)]
/// XEN_NETTXF_*
enum NetTxFlags {
    CsumBlank = 1 << 0,
    DataValidated = 1 << 1,
    MoreData = 1 << 2,
    ExtraInfo = 1 << 3,
}

#[repr(u16)]
#[derive(Debug)]
#[allow(dead_code)]
/// XEN_NETRXF_*
enum NetRxFlags {
    DataValidated = 1 << 0,
    CsumBlank = 1 << 1,
    MoreData = 1 << 2,
    ExtraInfo = 1 << 3,
    GsoPrefix = 1 << 4,
}

#[repr(C)]
#[derive(Debug)]
/// struct xen_netif_tx_request
struct NetifTxRequest {
    gref: GrantRef,
    offset: u16,
    flags: NetTxFlags,
    id: u16,
    size: u16,
}

#[repr(C)]
#[derive(Debug)]
/// struct xen_netif_tx_response
struct NetifTxResponse {
    id: u16,
    status: NetifRsp,
}

#[repr(C)]
#[derive(Debug)]
/// struct xen_netif_rx_request
struct NetifRxRequest {
    id: u16,
    gref: GrantRef,
}

#[repr(C)]
#[derive(Debug)]
/// struct xen_netif_rx_response
struct NetifRxResponse {
    id: u16,
    offset: u16,
    flags: NetRxFlags,
    status: i16,
}

type TxSharedRing = SharedRing<NetifTxRequest, NetifTxResponse>;
type RxSharedRing = SharedRing<NetifRxRequest, NetifRxResponse>;

type TxFrontRing = FrontRing<NetifTxRequest, NetifTxResponse>;
type RxFrontRing = FrontRing<NetifRxRequest, NetifRxResponse>;

/// Buffer allocated for packet reception
struct RxBuffer {
    pub id: u16,
    pub page: *mut u8,
    pub grant_ref: GrantRef,
}

/// Buffer allocated for packet transmission
struct TxBuffer {
    pub id: u16,
    pub pkt: Option<Packet>,
    pub grant_ref: GrantRef,
}

/// A Xen vif (virtual interface)
pub struct XenNetDevice {
    evtchn: EvtchnPort,
    backend_id: u16,
    tx_ring: TxFrontRing,
    rx_ring: RxFrontRing,
    tx_buffer: SpinLock<Vec<TxBuffer>>,
    rx_buffer: InterruptSpinLock<Vec<RxBuffer>>,
    intf: *const Arc<RwLock<Interface>>,
}
impl XenNetDevice {
    fn device_callback(_: EvtchnPort, data: *mut u8) {
        let xen_dev = unsafe { &mut (*(data as *mut XenNetDevice)) };

        xen_dev.rx_packet();
    }

    /// Creates a new Xen network device with id `id`.
    ///
    /// This interface will be the backend of the interface `intf`
    pub fn new(id: u32, intf: &Arc<RwLock<Interface>>) -> Result<Box<Self>, ()> {
        // Compute the root path that contains all the information for the
        // network device with id "id"
        let vif_root = format!("device/vif/{}", id);

        // Get the backend id. This is basically the domain id of the backend
        // It will be useful for event allocation and page grant
        let backend_path = try!(CString::new(format!("{}/backend-id", vif_root)));
        let backend_id = try!(XenStore::read_value::<u16>(backend_path));

        // Create RX and TX shared ring
        let mut tx_sring = try!(TxSharedRing::new(PAGE_SIZE).ok_or(()));
        let mut rx_sring = try!(RxSharedRing::new(PAGE_SIZE).ok_or(()));

        // Grant access to the rings to the backend
        let tx_ref = try!(tx_sring.grant_access(backend_id).ok_or(()));
        let rx_ref = try!(rx_sring.grant_access(backend_id).ok_or(()));

        let mut xen_dev = Box::new(XenNetDevice {
            evtchn: 0,
            backend_id: backend_id,
            tx_ring: TxFrontRing::new(tx_sring),
            rx_ring: RxFrontRing::new(rx_sring),
            tx_buffer: SpinLock::new(Vec::new()),
            rx_buffer: InterruptSpinLock::new(Vec::new()),
            intf: intf as *const _,
        });

        // This is legit as the event will be gone before the XenNetDevice
        let xen_dev_ptr = Box::into_raw(xen_dev);

        unsafe {
            xen_dev = Box::from_raw(xen_dev_ptr);
        }

        // Create a new event to receive interruptions about this device
        let evtchn = {
            try!(event::dispatcher().alloc_unbound(
                    backend_id, Self::device_callback,
                    xen_dev_ptr as *mut _).map_err(|_| ()
                 ))
        };

        xen_dev.evtchn = evtchn;

        // Start a transaction with the Xen store to finalize the device's
        // configuration
        let mut t = try!(XenStore::start_transaction());

        // Get the mac address as a String from the Xen Store
        let mac = try!(t.read(try!(CString::new(format!("{}/mac", vif_root)))));

        // Convert it to HwAddr
        let hw_addr = try!(HwAddr::from_str(mac));

        // Set tx and rx grant references
        try!(t.write(try!(CString::new(format!("{}/tx-ring-ref", vif_root))),
                     try!(CString::new(tx_ref.to_string()))));
        try!(t.write(try!(CString::new(format!("{}/rx-ring-ref", vif_root))),
                     try!(CString::new(rx_ref.to_string()))));

        // Set the event channel
        try!(t.write(try!(CString::new(format!("{}/event-channel", vif_root))),
                     try!(CString::new(evtchn.to_string()))));

        // Request packet transfer via flipping rather than copy. When using
        // flipping Xen backend driver will use paging to give us the packet
        // rather than copying the packet in the granted page
        try!(t.write(try!(CString::new(format!("{}/request-rx-copy", vif_root))),
                     CString::new("0").unwrap()));

        // Set the device as connected
        let state_path = try!(CString::new(format!("{}/state", vif_root)));

        try!(t.switch_state(state_path.clone(), XenbusState::Connected));

        try!(t.end());

        // Wait for the device to be connected
        // XXX: Add xen store watch implementation to avoid busy wait
        loop {
            let state: XenbusState = unsafe {
                mem::transmute(try!(XenStore::read_value::<u8>(state_path.clone())))
            };

            if state == XenbusState::Connected {
                break;
            } else if state < XenbusState::Connected {
                ::hal::xen::sched::yield_cpu();
                continue;
            } else {
                return Err(());
            }
        }

        // Unmask the event
        event::dispatcher().unmask_event(evtchn);

        // Set interface info
        *intf.write().name_mut() = format!("xen{}", id);
        *intf.write().hw_addr_mut() = hw_addr;

        Ok(xen_dev)
    }


    // Check for received packet. If there are packets enqueue them in
    // the network stack's rx queue to be processed.
    fn rx_packet(&mut self) {
    }
}
