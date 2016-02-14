use core::mem;
use core::slice;

use vec::Vec;
use string::String;

use thread::{Scheduler, WaitQueue};

use sync::spin::InterruptSpinLock;

use hal::arch::utils::wmb;

use hal::xen::event::{dispatcher, send};

use hal::xen::defs::XENSTORE_RING_SIZE;
use hal::xen::defs::{XenstoreInterface, EvtchnPort};

use hal::xen::store::{Result, Error};

const REQ_ID_COUNT: usize = 20;

pub struct RequestBuilder {
    msg_type: XsdSockmsgType,
    req_id: u32,
    tx_id: u32,
    data: Vec<u8>,
}

impl RequestBuilder {
    pub fn new(tx_id: u32) -> Self {
        RequestBuilder {
            msg_type: XsdSockmsgType::Debug,
            req_id: 0,
            tx_id: tx_id,
            data: Vec::new(),
        }
    }

    pub fn set_msg_type(mut self, t: XsdSockmsgType) -> Self {
        self.msg_type = t;
        self
    }

    pub fn append_data(mut self, data: &[u8]) -> Self {
        self.data.extend(data);
        self
    }

    pub fn set_req_id(mut self, req_id: u32) -> Self {
        self.req_id = req_id;
        self
    }

    pub fn build_xsdmsg(&self) -> XsdSockmsg {
        XsdSockmsg {
            msg_type: self.msg_type,
            req_id: self.req_id,
            tx_id: self.tx_id,
            len: self.data.len() as u32,
        }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

pub struct XenStoreImpl {
    interface: *mut XenstoreInterface,
    port: EvtchnPort,
    interface_notifier: WaitQueue,
    req_lock: InterruptSpinLock<()>,
    id_pool: RequestIdPool,
    req_pool: [Request; REQ_ID_COUNT],
}

unsafe impl Sync for XenStoreImpl {}
unsafe impl Send for XenStoreImpl {}

impl XenStoreImpl {
    pub fn new(interface: *mut XenstoreInterface, port: EvtchnPort) -> Self {
        XenStoreImpl {
            interface: interface,
            port: port,
            interface_notifier: WaitQueue::new(),
            req_lock: InterruptSpinLock::new(()),
            id_pool: RequestIdPool::new(),
            req_pool: [Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty(),
                       Request::empty()]
        }
    }

    pub fn str_to_err(s: String) -> Error {
        match s.as_ref() {
            "EINVAL" => Error::Inval,
            "EACCES" => Error::Acces,
            "EEXIST" => Error::Exist,
            "EISDIR" => Error::Isdir,
            "ENOENT" => Error::Noent,
            "ENOMEM" => Error::Nomem,
            "ENOSPC" => Error::Nospc,
            "EIO" => Error::Io,
            "ENOTEMPTY" => Error::Notempty,
            "ENOSYS" => Error::Nosys,
            "EROFS" => Error::Rofs,
            "EBUSY" => Error::Busy,
            "EAGAIN" => Error::Again,
            "EISCONN" => Error::Isconn,
            _ => Error::Unknown,
        }
    }

    fn xen_store_callback(_: EvtchnPort, data: *mut u8) {
        let store = unsafe { &mut *(data as *mut XenStoreImpl) };

        store.interface_notifier.unblock_all();
    }

    fn xen_store_thread() {
        use super::STORE;

        let mut store: &mut Self = &mut *STORE.as_mut();

        loop {
            // Dummy initialization
            let mut resp = XsdSockmsg::empty(0);

            // Wait for and read a XsdSockmsg
            store.read_response_bytes(resp.as_mut_bytes());

            let req_id = resp.req_id as usize;
            let size = resp.len as usize;

            // Store the message in the pool to be fetched by the waiting
            // thread
            store.req_pool[req_id].reply_msg = resp;

            let mut data = vec![0; size];

            if size > 0 {
                // Read extra data if necessary
                store.read_response_bytes(&mut data[..]);
            }

            // Store the data in the pool to be fetched by the waiting thread
            store.req_pool[req_id].reply_data = data;

            // Unblock the thread that was waiting for a response
            store.req_pool[req_id].wait.unblock();

            send(store.port);
        }
    }

    fn write_request_bytes(&mut self, b: &[u8]) {
        let mut lock;
        let mut prod;
        let interface_ref = unsafe { &mut *self.interface };

        prod = interface_ref.req_prod;

        loop {
            wait_event!(self.interface_notifier,
                        prod + b.len() as u32 - interface_ref.req_cons <=
                        XENSTORE_RING_SIZE as u32);

            lock = self.req_lock.lock();

            let tmp = prod + b.len() as u32 - interface_ref.req_cons;

            if  tmp <= XENSTORE_RING_SIZE as u32 {
                break
            }

            mem::drop(lock);
        }

        for i in 0..b.len() {
            let ring_index = prod & (XENSTORE_RING_SIZE as u32 - 1);

            interface_ref.req[ring_index as usize] = b[i];

            prod += 1;
        }

        wmb();
        interface_ref.req_prod = prod;

        mem::drop(lock);
    }

    fn read_response_bytes(&self, b: &mut [u8]) {
        let interface_ref = unsafe { &mut *self.interface };

        wait_event!(self.interface_notifier,
                    interface_ref.rsp_prod - interface_ref.rsp_cons >=
                    b.len() as u32);

        let mut cons = interface_ref.rsp_cons;

        for i in 0..b.len() {
            let ring_index = cons & (XENSTORE_RING_SIZE as u32 - 1);

            b[i] = interface_ref.rsp[ring_index as usize];

            cons += 1;
        }

        interface_ref.rsp_cons = cons;
    }

    pub fn init_event(&mut self) {
        dispatcher().bind_port(self.port, Self::xen_store_callback,
                               self as *mut _ as *mut u8);
        dispatcher().unmask_event(self.port);
        send(self.port);

        Scheduler::spawn(|| {
            Self::xen_store_thread();
        });
    }

    pub fn send(&mut self, mut req: RequestBuilder) -> Result<Vec<u8>> {
        let req_id = self.id_pool.alloc() as usize;

        req = req.set_req_id(req_id as u32);

        self.write_request_bytes(req.build_xsdmsg().as_bytes());
        self.write_request_bytes(&req.data()[..]);

        send(self.port);

        self.req_pool[req_id].wait.block();

        let mut data = self.req_pool[req_id].reply_data.clone();

        if data.last().map_or(false, |c| *c == 0) {
            data.pop();
        }

        let ret = match self.req_pool[req_id].reply_msg.msg_type {
            XsdSockmsgType::Error =>
                match String::from_utf8(data) {
                    Err(..) => Err(Error::Conversion),
                    Ok(s) => Err(XenStoreImpl::str_to_err(s)),
                },
            _ => Ok(data),
        };

        unsafe {
            self.id_pool.release(req_id as u32);
        }

        ret
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum XsdSockmsgType {
    Debug,
    Directory,
    Read,
    GetPerms,
    Watch,
    Unwatch,
    TransactionStart,
    TransactionEnd,
    Introduce,
    Release,
    GetDomainPath,
    Write,
    Mkdir,
    Remove,
    SetPerms,
    WatchEvent,
    Error,
    IsDomainIntroduced,
    Resume,
    SetTarget,
    Restrict,
    ResetWatches,
}

#[allow(dead_code)]
pub struct XsdSockmsg {
    msg_type: XsdSockmsgType,
    req_id: u32,
    tx_id: u32,
    len: u32,
}

impl XsdSockmsg {
    pub fn empty(req_id: u32) -> Self {
        XsdSockmsg {
            // Dummy
            msg_type: XsdSockmsgType::Debug,
            req_id: req_id,
            tx_id: Default::default(),
            len: Default::default(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const _ as *const u8,
                                  mem::size_of::<Self>())
        }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut _ as *mut u8,
                                      mem::size_of::<Self>())
        }
    }
}

struct Request {
    wait: WaitQueue,
    reply_msg: XsdSockmsg,
    reply_data: Vec<u8>,
}

impl Request {
    pub fn empty() -> Self {
        Request {
            wait: WaitQueue::new(),
            reply_msg: XsdSockmsg::empty(Default::default()),
            reply_data: Vec::new(),
        }
    }
}

struct RequestIdPool {
    pool: InterruptSpinLock<[bool; REQ_ID_COUNT]>,
    wait: WaitQueue,
}

impl RequestIdPool {
    pub fn new() -> Self {
        RequestIdPool {
            pool: InterruptSpinLock::new([true; REQ_ID_COUNT]),
            wait: WaitQueue::new(),
        }
    }

    fn has_free_id(&self) -> bool {
        let locked_pool = self.pool.lock();

        for i in 0..locked_pool.len() {
            if locked_pool[i] {
                return true;
            }
        }

        false
    }

    pub fn alloc(&mut self) -> u32 {
        let mut id;
        let mut locked_pool;

        'outer: loop {
            id = 0;

            wait_event!(self.wait, self.has_free_id());

            locked_pool = self.pool.lock();

            while id < locked_pool.len() {
                if locked_pool[id] {
                    break 'outer;
                }

                id += 1;
            }

            mem::drop(locked_pool);
        }

        locked_pool[id] = false;

        id as u32
    }

    pub unsafe fn release(&self, req_id: u32) {
        self.pool.lock()[req_id as usize] = true;

        self.wait.unblock();
    }
}
