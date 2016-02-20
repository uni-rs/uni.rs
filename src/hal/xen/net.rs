//! Implementation of Xen's network driver

use core::{mem, ptr};
use core::str::FromStr;

use alloc_uni::{__rust_allocate, __rust_deallocate};

use boxed::Box;

use vec::Vec;
use string::ToString;

use sync::spin::{InterruptSpinLock, SpinLock};

use ffi::CString;

use net::{Interface, InterfaceWeak, Packet, Stack as NetStack};
use net::defs::{Device, HwAddr};

use hal::mmu::{Vaddr, Mfn};

use hal::xen::store::{XenStore, XenbusState};
use hal::xen::grant::{Table as GrantTable, Ref as GrantRef};
use hal::xen::ring::{SharedRing, FrontRing, Idx as RingIdx};

use hal::xen::defs::EvtchnPort;

use hal::arch::defs::PAGE_SIZE;
use hal::arch::utils::{rmb, wmb};

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

/// Verify if a vif (virtual interface) with id `id` exists
fn vif_id_exists(id: u32) -> bool {
    let mut t = XenStore::start_transaction().unwrap();
    let path = CString::new(format!("device/vif/{}", id)).unwrap();

    let res = match t.read(path) {
        Ok(..) => true,
        Err(..) => false,
    };

    t.end().unwrap();

    res
}

/// Returns a list of interfaces that have a xen backend
pub fn discover() -> Vec<Interface> {
    let mut id = 0;
    let mut v = Vec::new();

    // Create interface for every `id` valid
    while vif_id_exists(id) {
        let interface = Interface::new();
        let interface_weak = interface.downgrade();

        v.push(interface);

        // Instantiate the Xen backend
        match XenNetDevice::new(id, interface_weak) {
            Ok(i) => {
                // Set it as pv_device of the interface
                v.last().unwrap().write().pv_device_set(i);
            }
            Err(..) => {
                v.pop();
                println!("Warning: Impossible to initialize xen network interface {}",
                         id);
            }
        }

        id += 1;
    }

    v
}

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
    intf: InterfaceWeak,
}

impl Device for XenNetDevice {
    fn refresh(&mut self) {
        self.refresh_tx_ring();
        self.refresh_rx_ring();
    }

    /// Transmit a packet over the network
    fn tx_packet(&mut self, pkt: Packet) {
        // Find a free TX buffer
        for b in &mut *self.tx_buffer.lock() {
            // Nope that one is taken
            if b.pkt.is_some() {
                continue;
            }

            let pkt_size = pkt.size();
            let pkt_offset = pkt.offset();

            // Grant access to the packet
            b.grant_ref.grant_access(self.backend_id,
                                     Mfn::from(Vaddr::from_ptr(pkt.page())),
                                     true);

            b.pkt = Some(pkt);

            {
                let index = self.tx_ring.req_prod() as usize;

                let req = unsafe {
                    self.tx_ring.sring_mut().request_from_index(index)
                };

                // Give the necessary information to the backend via a
                // tx_request
                req.gref = b.grant_ref.clone();
                req.offset = pkt_offset as u16;
                req.flags = NetTxFlags::DataValidated;
                req.id = b.id;
                req.size = pkt_size as u16;
            }

            // Update ring index
            unsafe {
                *self.tx_ring.req_prod_mut() += 1;
            }

            wmb();

            // Push the request in the shared ring and notify the backend if
            // necessary
            if self.tx_ring.push_requests() {
                event::send(self.evtchn);
            }

            break;
        }
    }
}

impl XenNetDevice {
    /// Callback handling Xen network events
    fn device_callback(_: EvtchnPort, data: *mut u8) {
        let xen_dev = unsafe { &mut (*(data as *mut XenNetDevice)) };

        xen_dev.rx_packet();
    }

    /// Creates a new Xen network device with id `id`.
    ///
    /// This interface will be the backend of the interface `intf`
    pub fn new(id: u32, intf: InterfaceWeak) -> Result<Box<Self>, ()> {
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
            intf: intf,
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
        let hw_addr = try!(HwAddr::from_str(&mac));

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

        // Init buffers
        try!(xen_dev.init_rx());
        try!(xen_dev.init_tx());

        // Unmask the event
        event::dispatcher().unmask_event(evtchn);

        let parent = xen_dev.intf.upgrade().unwrap();

        // Set interface info
        *parent.write().name_mut() = format!("xen{}", id);
        *parent.write().hw_addr_mut() = hw_addr;

        Ok(xen_dev)
    }

    fn init_rx(&mut self) -> Result<(), ()> {
        let mut v: Vec<RxBuffer> = Vec::with_capacity(self.rx_ring.size());

        // We populate the rx ring with RxRequest. These requests give to the
        // backend some pages to work with in order to send us some packets
        //
        // The way TX works with Xen is as follow:
        //
        // 1. We put RxRequests in the ring. These requests contain an
        // allocated and granted page for the backend to work with
        //
        // 2. The backend receives a packet for us
        //
        // 3. The backend pops a RxRequest and use the granted page to transfer
        // use the package
        //
        // 4. The backend pushes a RxResponse in the ring
        //
        // 5. We process this RxResponse, ungrant the page and enqueue the
        // packet in the Rx queue to be treated by the network stack
        //
        // This is why we need to keep trap of the granted page
        for i in 0..self.rx_ring.size() {
            let page = __rust_allocate(PAGE_SIZE, PAGE_SIZE);

            if page.is_null() {
                return Err(());
            }

            // Grant access to the page to the backend
            let grant = try!(GrantTable::alloc_ref().ok_or(()));

            grant.grant_access(self.backend_id,
                               Mfn::from(Vaddr::from_ptr(page)), false);

            // Tell the backend about the new buffer via a RxRequest
            let req = unsafe { self.rx_ring.sring_mut().request_from_index(i) };

            req.id = i as u16;
            req.gref = grant.clone();

            // Keep track of buffers internally
            v.push(RxBuffer {
                id: i as u16,
                page: page,
                grant_ref: grant,
            });
        }

        // Update ring's request production index
        unsafe {
            *self.rx_ring.req_prod_mut() += self.rx_ring.size() as RingIdx;
        }

        // Notify the backend
        if self.rx_ring.push_requests() {
            event::send(self.evtchn);
        }

        unsafe {
            // Set notification index so that the backend notifies us when it
            // pushes new responses in the ring
            let rsp_event = self.rx_ring.rsp_cons() + 1;

            self.rx_ring.sring_mut().rsp_event_set(rsp_event);
        }

        self.rx_buffer = InterruptSpinLock::new(v);

        Ok(())
    }

    fn init_tx(&mut self) -> Result<(), ()> {
        let mut v: Vec<TxBuffer> = Vec::with_capacity(self.tx_ring.size());

        // We internally initialize empty buffers.
        // The way TX works with Xen is as follow:
        //
        // 1. We sent a TxRequest with a page that contains the packet to
        // transmit. The backend has granted access to the page.
        //
        // 2. The backend will transmit our packet and push in the ring a
        // TxResponse
        //
        // 3. refresh_tx() is called
        //
        // 4. refresh_tx() will consume TxResponse(s), ungrant access to the
        // page and free the packet.
        //
        // This is why we need to keep track of the grant reference and the
        // packet so that refresh_tx() can do its job.
        for i in 0..self.tx_ring.size() {
            let grant = try!(GrantTable::alloc_ref().ok_or(()));

            let buffer = TxBuffer {
                id: i as u16,
                pkt: None,
                grant_ref: grant,
            };

            v.push(buffer);
        }

        self.tx_buffer = SpinLock::new(v);

        Ok(())
    }

    /// Checks for received packet
    ///
    /// If there are packets, they are enqueued in the network stack's rx queue
    /// to be processed by the network thread
    fn rx_packet(&mut self) {
        loop {
            let prod = unsafe { self.rx_ring.sring_mut().rsp_prod() };
            let mut cons = self.rx_ring.rsp_cons();

            rmb();

            // Iterate through the RxResponses inside the ring
            while cons != prod {
                let resp = unsafe {
                    self.rx_ring.sring_mut()
                                   .response_from_index(cons as usize)
                };

                let id = resp.id as usize;

                {
                    let mut rx_buffer_locked = self.rx_buffer.lock();

                    // End access to the page now that we have the packet
                    rx_buffer_locked[id].grant_ref.end_access();

                    // If everything is good, enqueue the packet in the rx
                    // queue of the network stack
                    if resp.status > NetifRsp::Null as i16 {
                        let mut pkt = unsafe {
                            Packet::new(rx_buffer_locked[id].page,
                                        resp.offset as usize,
                                        resp.status as usize)
                        };

                        pkt.set_interface(self.intf.clone());

                        NetStack::instance().enqueue_rx_packet(pkt);
                    } else {
                        // Deallocate the page if an error occurred
                        __rust_deallocate(rx_buffer_locked[id].page, PAGE_SIZE,
                                          PAGE_SIZE);
                    }

                    // Set the page as null so that we now that this buffer
                    // entry is free to re-use when refresh_rx_buffers() is
                    // called
                    rx_buffer_locked[id].page = ptr::null_mut();
                }

                cons += 1;
            }

            unsafe {
                *self.rx_ring.rsp_cons_mut() = cons;
            }

            if !self.rx_ring.final_check_for_responses() {
                break;
            }
        }
    }

    /// Give new rx buffers to the backend to replace those that were used
    fn refresh_rx_ring(&mut self) {
        let mut count = 0;
        let mut rx_buffer_locked = self.rx_buffer.lock();

        for b in &mut *rx_buffer_locked {
            // RxBuffer is in use
            if !b.page.is_null() {
                continue;
            }

            // Allocate new buffer
            b.page = __rust_allocate(PAGE_SIZE, PAGE_SIZE);

            // OOM: Ignore buffer reload for now
            if b.page.is_null() {
                break;
            }

            // Grant read/write access to the backed
            b.grant_ref.grant_access(self.backend_id,
                                     Mfn::from(Vaddr::from_ptr(b.page)),
                                     false);

            let prod = self.rx_ring.req_prod() as usize;
            let req = unsafe {
                self.rx_ring.sring_mut().request_from_index(prod)
            };

            // Push a new request to inform the backend that it can use a new
            // buffer
            req.id = b.id;
            req.gref = b.grant_ref.clone();

            count += 1;
        }

        // Update ring's prod index
        unsafe {
            *self.rx_ring.req_prod_mut() += count;
        }

        mem::drop(rx_buffer_locked);

        // Push requests and notify the backend if necessary
        if self.rx_ring.push_requests() {
            event::send(self.evtchn);
        }
    }

    fn refresh_tx_ring(&mut self) {
        let prod = unsafe { self.tx_ring.sring_mut().rsp_prod() };

        rmb();

        let mut cons = self.tx_ring.rsp_cons();

        while cons != prod {
            let resp = unsafe {
                self.tx_ring.sring_mut().response_from_index(cons as usize)
            };

            if resp.status == NetifRsp::Null {
                continue;
            }

            {
                let id = resp.id;
                let mut tx_buffer = self.tx_buffer.lock();

                tx_buffer[id as usize].grant_ref.end_access();

                mem::drop(tx_buffer[id as usize].pkt.take());
            }

            cons += 1;
        }

        unsafe {
            *self.tx_ring.rsp_cons_mut() = prod;
        }
    }
}
