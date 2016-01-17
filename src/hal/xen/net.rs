//! Implementation of Xen's network driver

use sync::Arc;

use vec::Vec;

use sync::spin::{InterruptSpinLock, SpinLock, RwLock};

use net::{Interface, Packet};

use hal::xen::grant::Ref as GrantRef;
use hal::xen::ring::{SharedRing, FrontRing};

use hal::xen::defs::EvtchnPort;

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
