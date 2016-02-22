//! Definitions of types and traits related to packet filtering
//!
//! Two different parameters class are going to be mentioned in this module:
//!
//! - Generic parameter: ether type, protocol id, ...
//! - Specific parameter: hardware address, ip address, ...

use core::marker::PhantomData;

use sync::Arc;

use boxed::Box;
use btree_map::BTreeMap;

use net::Packet;

use net::defs::Rule;

use net::conn::MultiConn;

/// Callbacks used by `SpecificFilter`.
///
/// **FIXME**: I feel like this is somehow hacky
pub trait SpecificCallbacks<T: Ord + Clone> {
    /// Determine if a rule has an upper protocol component.
    ///
    /// For example if you implement a network protocol this will return `true`
    /// if the rule has a transport layer component.
    fn has_upper_filter(rule: &Rule) -> bool;

    /// Create a new generic filter that filters the generic parameter `param`
    /// of type T.
    fn filter_from_generic_parameter(param: T) -> Option<Box<GenericFilterTrait>>;

    /// Set the component of the `rule` implemented by this protocol based on
    /// information contained in a packet.
    fn set_layer_rule(rule: &mut Rule, pkt: &Packet);
}

/// Extract generic/specific parameters of type T from a packet or a rule.
///
/// This is used by filters to extract the generic/specific parameter and use
/// it to route the packet to the appropriate connexion (if such connexion
/// exists).
pub trait Extractor<T> {
    /// Extract the parameter from a rule.
    ///
    /// This will return `None` if the method fails to extract the parameter.
    fn from_rule(rule: &Rule) -> Option<T>;

    /// Extract the parameter from a Packet.
    ///
    /// This will return `None` if the method fails to extract the parameter.
    fn from_packet(pkt: &Packet) -> Option<T>;
}

/// Sanitize an incoming packet.
///
/// Packets that arrive from the network need to be checked by every used
/// protocols for correctness (i.e. checksum, ...).
///
/// Some properties or extra information might need to be added to the packet
/// as well.
///
/// This trait let protocols do this.
pub trait PacketSanitizer {
    /// This method is called when a filter needs to sanitize a packet.
    ///
    /// If this method returns an `Err` the packet will be dropped.
    fn sanitize(pkt: &mut Packet) -> Result<(), ()>;
}

/// Trait implemented by generic filters (i.e. filters based on generic
/// parameters)
///
/// Generic filters allow protocols to route packets to the appropriate
/// connexion based on a generic parameter.
///
/// It may drop the packet if no matching connexion exists or if the packet is
/// invalid (failed checksum, ...).
pub trait GenericFilterTrait {
    /// Insert a new multi connexion to the filter based on a rule.
    fn insert_multi(&mut self, conn: Arc<MultiConn>,
                    rule: &Rule) -> Result<(), ()>;

    /// Filter and route an incoming packet to a connexion (uni or multi).
    fn rx(&self, pkt: Packet);

    /// Filter and route an incoming packet to a multi connexion
    fn rx_multi(&self, pkt: Packet, rule: Rule);
}

/// Trait implemented by specific filters (i.e. filters based on specific
/// parameters)
///
/// Specific filters allow protocols to route packets to the appropriate
/// connexion based on a specific parameter
///
/// It may drop the packet if no matching connexion exists.
pub trait SpecificFilterTrait<T> {
    /// Create a new specific filter.
    ///
    /// The generic parameter taken as parameter is used to associate a
    /// specific filter with a generic one.
    fn new(generic_param: T) -> Self;

    /// Insert a new multi connexion to the filter based on a rule.
    fn insert_multi(&mut self, conn: Arc<MultiConn>,
                    rule: &Rule) -> Result<(), ()>;

    /// Filter and route an incoming packet to a connexion (uni or multi).
    fn rx(&self, pkt: Packet);

    /// Filter and route an incoming packet to a multi connexion
    fn rx_multi(&self, pkt: Packet, rule: Rule);
}

/// Filter packets on a generic parameter
///
/// This is a generic class that can be used by protocols to implement a
/// generic filter.
///
/// Here is the explanation of types taken as parameters:
///
/// - `T`: type of the generic parameter.
/// - `U`: type of the filter used to filter on specific parameter.
/// - `E`: type that allows extraction of the generic parameter from rules and
///        packets.
/// - `S`: type that allows the protocol to sanitize incoming packets.
pub struct GenericFilter<T, U, E, S> where T: Ord + Clone,
                                           U: SpecificFilterTrait<T>,
                                           E: Extractor<T>,
                                           S: PacketSanitizer {
    filters: BTreeMap<T, U>,
    _extractor: PhantomData<E>,
    _sanitizer: PhantomData<S>,
}

impl<T, U, E, S> GenericFilter<T, U, E, S> where T: Ord + Clone,
                                                 U: SpecificFilterTrait<T>,
                                                 E: Extractor<T>,
                                                 S: PacketSanitizer {
    /// Create a new generic filter
    pub fn new() -> Self {
        GenericFilter {
            filters: BTreeMap::new(),
            _extractor: PhantomData,
            _sanitizer: PhantomData,
        }
    }
}

impl<T, U, E, S> GenericFilterTrait for GenericFilter<T, U, E, S>
                                    where T: Ord + Clone,
                                          U: SpecificFilterTrait<T>,
                                          E: Extractor<T>,
                                          S: PacketSanitizer {
    fn insert_multi(&mut self, conn: Arc<MultiConn>,
                    rule: &Rule) -> Result<(), ()> {
        // Extract the generic parameter of the rule. If none exists then it's
        // an error and the connexion cannot be added
        if let Some(key) = E::from_rule(rule) {
            // Check if a filter already exists, if none add one
            if !self.filters.contains_key(&key) {
                self.filters.insert(key.clone(), U::new(key.clone()));
            }

            // Insert the connexion in the specific filter
            self.filters.get_mut(&key).unwrap().insert_multi(conn, rule)
        } else {
            Err(())
        }
    }

    fn rx(&self, mut pkt: Packet) {
        // Sanitize the packet
        if S::sanitize(&mut pkt).is_ok() {
            // Get the generic parameter
            if let Some(key) = E::from_packet(&pkt) {
                // Pass the packet to the specific filter if such filter exists
                if let Some(filter) = self.filters.get(&key) {
                    filter.rx(pkt);
                }
            }
        }
    }

    fn rx_multi(&self, mut pkt: Packet, rule: Rule) {
        // This basically behaves like rx() except that it give the packet
        // to the multi in the filter
        if S::sanitize(&mut pkt).is_ok() {
            if let Some(key) = E::from_packet(&pkt) {
                if let Some(filter) = self.filters.get(&key) {
                    filter.rx_multi(pkt, rule);
                }
            }
        }
    }
}
