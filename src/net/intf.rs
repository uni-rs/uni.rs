use boxed::Box;
use string::String;

use sync::{Arc, Weak};

use sync::spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use net::{Instance, InstanceWeak, Packet, PacketBuilder};

use net::defs::{Rule, HwAddr, Ipv4Addr, Device};

use hal::net::HwInterface;

// XXX: Should this be in net::defs ?
/// IPv4 configuration of an interface
pub struct V4Configuration {
    /// Main IPv4 address
    pub ipv4: Ipv4Addr,
    /// Subnet mask
    pub ipv4_mask: Ipv4Addr,
    /// Gateway IPv4 address
    pub ipv4_gateway: Ipv4Addr,
}

#[derive(Clone)]
/// A shareable network interface.
///
/// This is backed by an Arc and a RwLock so that accesses are safe and the
/// entity can be shared.
pub struct Interface(Arc<RwLock<InterfaceRaw>>);

unsafe impl Send for Interface {}

#[derive(Clone)]
/// A interface weak reference
pub struct InterfaceWeak(Weak<RwLock<InterfaceRaw>>);

unsafe impl Send for InterfaceWeak {}

/// The raw network interface
pub struct InterfaceRaw {
    /// The network stack the interface belongs to
    instance: InstanceWeak,
    /// Name of the interface
    name: String,
    /// Hardware address of the interface
    hw_addr: HwAddr,
    /// IPv4 configuration of the interface
    conf: V4Configuration,
    /// Underlying driver
    pv_device: Option<Box<HwInterface>>,
}

impl Interface {
    /// Creates a new network interface
    pub fn new(instance: &Instance) -> Self {
        let inner = InterfaceRaw {
            instance: instance.downgrade(),
            name: String::new(),
            hw_addr: HwAddr::empty(),
            conf: V4Configuration {
                ipv4: Ipv4Addr::new(0, 0, 0, 0),
                ipv4_mask: Ipv4Addr::new(0, 0, 0, 0),
                ipv4_gateway: Ipv4Addr::new(0, 0, 0, 0),
            },
            pv_device: None,
        };

        Interface(Arc::new(RwLock::new(inner)))
    }

    /// Get a weak reference over the interface
    pub fn downgrade(&self) -> InterfaceWeak {
        InterfaceWeak(Arc::downgrade(&self.0))
    }

    /// Lock the interface with shared read access
    ///
    /// For more info see spin::RwLock::read()
    pub fn read(&self) -> RwLockReadGuard<InterfaceRaw> {
        self.0.read()
    }

    /// Lock the interface with exclusive write access
    ///
    /// For more info see spin::RwLock::write()
    pub fn write(&self) -> RwLockWriteGuard<InterfaceRaw> {
        self.0.write()
    }

    /// Receive a packet on the interface
    pub fn rx_packet(&self, _pkt: Packet) {
        unimplemented!();
    }

    /// Transmit a packet through the interface
    pub fn tx_packet(&self, _builder: PacketBuilder,
                     _rule: &Rule) -> Result<(), ()> {
        unimplemented!();
    }
}

impl InterfaceWeak {
    /// Upgrade the weak reference to a real reference
    pub fn upgrade(&self) -> Option<Interface> {
        self.0.upgrade().map(Interface)
    }
}

impl InterfaceRaw {
    #[inline]
    /// Returns a reference over the name of the interface
    pub fn name_ref(&self) -> &str {
        &self.name
    }

    #[inline]
    /// Returns a reference over the hardware of the interface
    pub fn hw_addr_ref(&self) -> &HwAddr {
        &self.hw_addr
    }

    #[inline]
    /// Returns a reference over the IPv4 configuration of the interface
    pub fn v4_configuration_ref(&self) -> &V4Configuration {
        &self.conf
    }

    #[inline]
    /// Returns a mutable reference over the name of the interface
    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    #[inline]
    /// Returns a mutable reference over the hardware of the interface
    pub fn hw_addr_mut(&mut self) -> &mut HwAddr {
        &mut self.hw_addr
    }

    #[inline]
    /// Returns a mutable reference over the IPv4 configuration of the interface
    pub fn v4_configuration_mut(&mut self) -> &mut V4Configuration {
        &mut self.conf
    }

    #[inline]
    #[doc(hidden)]
    pub fn pv_device_set(&mut self, pv: Box<HwInterface>) {
        self.pv_device = Some(pv);
    }

    #[inline]
    /// Refresh underlying driver
    pub fn refresh(&mut self) {
        self.pv_device.as_mut().unwrap().refresh();
    }
}
